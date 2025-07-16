use axum::{
    extract::{Path, Query, State, Request, Multipart},
    http::{header, StatusCode},
    response::{Html, Json, Response, Redirect},
};
use utoipa::OpenApi;

use crate::{
    dto::asset::{CreateAssetRequest, UpdateAssetRequest, ErrorResponse, FilterParams, AssetResponse, AssetListResponse, FilteredAssetResponse, CreateSubtitleRequest, CreateYouTubeLinkRequest, SubtitleResponse, YouTubeLinkResponse},
    services::asset::AssetService,
    middleware::auth::get_current_user,
    AppState,
};

/// ReengKi Admin API Documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        create_asset,
        list_asset,
        get_filtered_assets,
        update_asset,
        delete_asset
    ),
    components(
        schemas(CreateAssetRequest, UpdateAssetRequest, AssetResponse, AssetListResponse, FilteredAssetResponse, ErrorResponse, FilterParams, CreateSubtitleRequest, CreateYouTubeLinkRequest, SubtitleResponse, YouTubeLinkResponse)
    ),
    tags(
        (name = "assets", description = "Asset management operations"),
        (name = "admin", description = "Admin operations")
    ),
    info(
        title = "ReengKi Admin API",
        version = "1.0.0",
        description = "REST API for ReengKi educational asset management system",
        contact(
            name = "API Support",
            email = "support@reengki.com"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Development server")
    )
)]
pub struct ApiDoc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;


pub async fn dashboard_page(request: Request) -> Result<Html<&'static str>, Redirect> {
    // Check if user is authenticated and has admin role
    if let Some(user) = get_current_user(&request) {
        if user.can_access_admin() {
            return Ok(Html(include_str!("../templates/admin-head/dashboard-main.html")));
        }
    }
    
    // Redirect to login if not authenticated or not authorized
    Err(Redirect::permanent("/login"))
}

pub async fn asset_management_page(request: Request) -> Result<Html<&'static str>, Redirect> {
    // Check if user is authenticated and has admin role
    if let Some(user) = get_current_user(&request) {
        if user.can_access_admin() {
            return Ok(Html(include_str!("../templates/admin-head/dashboard-asset.html")));
        }
    }
    // Redirect to login if not authenticated or not authorized
    Err(Redirect::permanent("/login"))
}

pub async fn upload_file(
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    tracing::info!("File upload request received");
    let mut file_path = String::new();
    let mut file_type = String::new();
    let mut original_filename: Option<String> = None;
    let mut curriculum: Option<String> = None;
    let mut month: Option<String> = None;
    
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            let filename = field.file_name().unwrap_or("unknown").to_string();
            let content_type = field.content_type().unwrap_or("").to_string();
            let data = field.bytes().await.unwrap();
            
            tracing::info!("Processing file: {} (type: {}, size: {} bytes)", filename, content_type, data.len());
            original_filename = Some(filename.clone());
            
            // Check if file type is allowed
            let allowed_types = vec!["image/jpeg", "image/jpg", "image/png", "image/webp"];
            if !allowed_types.contains(&content_type.as_str()) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "INVALID_FILE_TYPE".to_string(),
                        message: format!("File type {} not allowed", content_type),
                    }),
                ));
            }
            
            // For now, save to a temporary upload folder and return the path
            // The actual curriculum/month/cover or curriculum/month/thumbnail path will be determined by the frontend
            let upload_dir = "asset/uploads";
            tokio::fs::create_dir_all(&upload_dir).await.unwrap();
            
            let safe_filename = sanitize_filename(&filename);
            let full_path = format!("{}/{}", upload_dir, safe_filename);
            
            // Save file to temporary location
            let mut file = File::create(&full_path).await.unwrap();
            file.write_all(&data).await.unwrap();
            
            tracing::info!("File saved to: {}", full_path);
            
            // Return path relative to asset folder for consistency with existing data
            file_path = format!("/asset/uploads/{}", safe_filename);
            file_type = content_type;
        } else if name == "curriculum" {
            let text = field.text().await.unwrap_or_default();
            curriculum = Some(text);
        } else if name == "month" {
            let text = field.text().await.unwrap_or_default();
            month = Some(text);
        }
    }
    
    if file_path.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "NO_FILE_UPLOADED".to_string(),
                message: "No file was uploaded".to_string(),
            }),
        ));
    }
    
    Ok(Json(serde_json::json!({
        "success": true,
        "file_path": file_path,
        "file_type": file_type,
        "message": "File uploaded successfully"
    })))
}

fn sanitize_filename(filename: &str) -> String {
    // Remove potentially dangerous characters
    let safe_chars: String = filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    
    // Add timestamp to prevent conflicts
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    format!("{}_{}", timestamp, safe_chars)
}

fn pretty_json_response<T: serde::Serialize>(data: T) -> Response {
    match serde_json::to_string_pretty(&data) {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json.into())
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Failed to serialize response".into())
            .unwrap(),
    }
}

#[utoipa::path(
    post,
    path = "/api/assets",
    tag = "assets",
    request_body = CreateAssetRequest,
    responses(
        (status = 200, description = "Asset created successfully", body = AssetResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn create_asset(
    State(state): State<AppState>,
    Json(request): Json<CreateAssetRequest>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    match AssetService::create_asset(&state.db, request).await {
        Ok(asset) => Ok(pretty_json_response(asset)),
        Err(e) => {
            tracing::error!("Failed to create asset: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "CREATE_FAILED".to_string(),
                    message: "Failed to create asset".to_string(),
                }),
            ))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/assets",
    tag = "assets",
    responses(
        (status = 200, description = "Assets retrieved successfully", body = AssetListResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn list_asset(
    State(state): State<AppState>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    match AssetService::list_asset(&state.db).await {
        Ok(response) => Ok(pretty_json_response(response)),
        Err(e) => {
            tracing::error!("Failed to list assets: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "LIST_ASSETS_FAILED".to_string(),
                    message: "Failed to retrieve assets list".to_string(),
                }),
            ))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/assets/filter",
    tag = "assets",
    params(FilterParams),
    responses(
        (status = 200, description = "Filtered assets retrieved successfully", body = FilteredAssetResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn get_filtered_assets(
    State(state): State<AppState>,
    Query(params): Query<FilterParams>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    match AssetService::get_filtered_assets(&state.db, params).await {
        Ok(data) => Ok(pretty_json_response(data)),
        Err(e) => {
            tracing::error!("Failed to get filtered assets: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "GET_FILTERED_ASSETS_FAILED".to_string(),
                    message: "Failed to retrieve filtered assets".to_string(),
                }),
            ))
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/assets/{asset_id}",
    tag = "assets",
    params(
        ("asset_id" = String, Path, description = "Asset ID to update")
    ),
    request_body = UpdateAssetRequest,
    responses(
        (status = 200, description = "Asset updated successfully", body = AssetResponse),
        (status = 404, description = "Asset not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn update_asset(
    State(state): State<AppState>,
    Path(asset_id): Path<String>,
    Json(request): Json<UpdateAssetRequest>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    match AssetService::update_asset(&state.db, &asset_id, request).await {
        Ok(asset) => Ok(pretty_json_response(asset)),
        Err(e) => {
            tracing::error!("Failed to update asset: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "UPDATE_FAILED".to_string(),
                    message: "Failed to update asset".to_string(),
                }),
            ))
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/assets/{asset_id}",
    tag = "assets",
    params(
        ("asset_id" = String, Path, description = "Asset ID to delete")
    ),
    responses(
        (status = 204, description = "Asset deleted successfully"),
        (status = 404, description = "Asset not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn delete_asset(
    State(state): State<AppState>,
    Path(asset_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match AssetService::delete_asset(&state.db, &asset_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: format!("Asset with id {} not found", asset_id),
            }),
        )),
        Err(e) => {
            tracing::error!("Failed to delete asset: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "DELETE_FAILED".to_string(),
                    message: "Failed to delete asset".to_string(),
                }),
            ))
        }
    }
}
