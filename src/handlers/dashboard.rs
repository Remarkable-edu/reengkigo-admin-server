use axum::{
    extract::{Multipart, State, Query, Path},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use tracing::{error, info};
use crate::{
    dto::asset::{CreateAssetResponse, AssetListResponse, AssetInfo, YouTubeLink, AssetFilterQuery, AssetFilterResponse, UpdateAssetRequest},
    AppState,
};

pub async fn dashboard_main() -> Html<&'static str> {
    Html(include_str!("../templates/admin-head/dashboard-main.html"))
}

pub async fn dashboard_asset() -> Html<&'static str> {
    Html(include_str!("../templates/admin-head/dashboard-asset.html"))
}


pub async fn create_asset(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let file_service = &app_state.file_service;
    let mut book_id = String::new();
    let mut title = String::new();
    let mut category = String::new();
    let mut files = Vec::new();
    let mut subtitles_json = String::new();

    // Parse multipart data
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        match field.name().unwrap_or("") {
            "book_id" => book_id = field.text().await.unwrap_or_default(),
            "title" => title = field.text().await.unwrap_or_default(),
            "category" => category = field.text().await.unwrap_or_default(),
            "subtitles" => subtitles_json = field.text().await.unwrap_or_default(),
            "cover_image" | "video_file" => {
                let filename = field.file_name().unwrap_or("unknown").to_string();
                if let Ok(data) = field.bytes().await {
                    // Check file size limit (500MB)
                    if data.len() > 500 * 1024 * 1024 {
                        return (
                            StatusCode::PAYLOAD_TOO_LARGE,
                            Json(CreateAssetResponse {
                                success: false,
                                asset_id: None,
                                message: format!("파일이 너무 큽니다: {:.2}MB", data.len() as f64 / (1024.0 * 1024.0)),
                                cover_image_url: None,
                                video_url: None,
                            })
                        ).into_response();
                    }
                    files.push((filename, data));
                }
            }
            _ => {}
        }
    }

    // Validate required fields
    if book_id.is_empty() || title.is_empty() || files.len() < 2 {
        return (
            StatusCode::BAD_REQUEST,
            Json(CreateAssetResponse {
                success: false,
                asset_id: None,
                message: "필수 필드 누락: 교재 ID, 제목, 표지 이미지, 비디오 파일".to_string(),
                cover_image_url: None,
                video_url: None,
            })
        ).into_response();
    }

    // Rename files and validate types
    let mut renamed_files = Vec::new();
    let mut has_image = false;
    let mut has_video = false;

    for (original_filename, data) in files {
        let extension = original_filename.rfind('.').map(|i| &original_filename[i..]).unwrap_or("");
        let new_filename = format!("{}{}", title, extension);
        
        let lower = new_filename.to_lowercase();
        if lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
            has_image = true;
        } else if lower.ends_with(".mp4") || lower.ends_with(".mov") || lower.ends_with(".avi") {
            has_video = true;
        }
        
        renamed_files.push((new_filename, data));
    }

    if !has_image || !has_video {
        return (
            StatusCode::BAD_REQUEST,
            Json(CreateAssetResponse {
                success: false,
                asset_id: None,
                message: "이미지 파일과 비디오 파일이 모두 필요합니다".to_string(),
                cover_image_url: None,
                video_url: None,
            })
        ).into_response();
    }

    let full_path = format!("{}/{}/", book_id, title);
    
    match file_service.upload_file(renamed_files, None, &full_path).await {
        Ok(response) => {
            let cover_image_url = response.uploaded.iter()
                .find(|f| f.filename.to_lowercase().contains(".png") || 
                          f.filename.to_lowercase().contains(".jpg") || 
                          f.filename.to_lowercase().contains(".jpeg"))
                .map(|f| f.url.clone());
                
            let video_url = response.uploaded.iter()
                .find(|f| f.filename.to_lowercase().contains(".mp4") || 
                          f.filename.to_lowercase().contains(".mov") || 
                          f.filename.to_lowercase().contains(".avi"))
                .map(|f| f.url.clone());

            info!("Asset created successfully: {} - {}", book_id, title);
            
            (
                StatusCode::OK,
                Json(CreateAssetResponse {
                    success: true,
                    asset_id: Some(format!("{}_{}", book_id, title)),
                    message: "에셋이 성공적으로 생성되었습니다".to_string(),
                    cover_image_url,
                    video_url,
                })
            ).into_response()
        }
        Err(err) => {
            error!("Asset creation failed: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CreateAssetResponse {
                    success: false,
                    asset_id: None,
                    message: format!("에셋 생성 실패: {}", err),
                    cover_image_url: None,
                    video_url: None,
                })
            ).into_response()
        }
    }
}

// Mock data for assets (in a real app, this would come from a database)
fn get_mock_assets() -> Vec<AssetInfo> {
    vec![
        AssetInfo {
            id: "J1R_Amazing_Animals".to_string(),
            curriculum: "J1R".to_string(),
            month: "Amazing Animals".to_string(),
            book_id: "J1R".to_string(),
            covers: vec!["cover/1_J1R.png".to_string(), "cover/2_J1R.png".to_string()],
            subtitles: vec![],
            youtube_links: vec![
                YouTubeLink {
                    thumbnail_file: "thumbnail/J1R_amazing.png".to_string(),
                    youtube_url: "https://youtu.be/v-r7AtCFc-w".to_string(),
                    title: Some("J1R Amazing".to_string()),
                }
            ],
            video_url: Some("/assets/J1R/Amazing_Animals/Amazing_Animals.mp4".to_string()),
        },
    ]
}

pub async fn list_assets() -> impl IntoResponse {
    info!("Listing all assets");
    
    let assets = get_mock_assets();
    
    (
        StatusCode::OK,
        Json(AssetListResponse { assets })
    ).into_response()
}

pub async fn filter_assets(
    Query(params): Query<AssetFilterQuery>
) -> impl IntoResponse {
    info!("Filtering assets with params: {:?}", params);
    
    let mut assets = get_mock_assets();
    
    // Filter by book_id if provided
    if let Some(book_id) = params.book_id {
        assets.retain(|asset| asset.book_id.contains(&book_id));
    }
    
    let total_found = assets.len();
    
    (
        StatusCode::OK,
        Json(AssetFilterResponse { assets, total_found })
    ).into_response()
}

pub async fn update_asset(
    Path(asset_id): Path<String>,
    Json(update_request): Json<UpdateAssetRequest>
) -> impl IntoResponse {
    info!("Updating asset: {} with request: {:?}", asset_id, update_request);
    
    // In a real app, this would update the asset in the database
    // For now, just return success
    
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Asset updated successfully"
        }))
    ).into_response()
}

pub async fn delete_asset(
    Path(asset_id): Path<String>
) -> impl IntoResponse {
    info!("Deleting asset: {}", asset_id);
    
    // In a real app, this would delete the asset from the database
    // For now, just return success
    
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Asset deleted successfully"
        }))
    ).into_response()
}