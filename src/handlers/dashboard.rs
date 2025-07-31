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

async fn get_assets(app_state: &AppState, book_id_filter: Option<&str>) -> Result<Vec<AssetInfo>, String> {
    use std::collections::HashMap;
    
    // 공통 파일 정보 구조체
    #[derive(Debug)]
    struct FileInfo {
        key: String,
        size: u64,
    }
    
    let files: Vec<FileInfo> = if let Some(book_id) = book_id_filter {
        // 특정 book_id로 필터링
        let folder_result = app_state.file_service.get_folder_files(None, book_id).await
            .map_err(|e| format!("Failed to get folder files: {}", e))?;
        folder_result.files.into_iter().map(|f| FileInfo {
            key: f.key,
            size: f.size,
        }).collect()
    } else {
        // 모든 파일 가져오기
        let all_files_result = app_state.file_service.get_all_files(None).await
            .map_err(|e| format!("Failed to get all files: {}", e))?;
        all_files_result.files.into_iter().map(|f| FileInfo {
            key: f.key,
            size: f.size,
        }).collect()
    };
    
    // {커리큘럼}/{제목}/{파일명} 패턴 필터링 및 그룹화
    let mut asset_groups: HashMap<String, Vec<FileInfo>> = HashMap::new();
    
    for file in files {
        let path_parts: Vec<&str> = file.key.split('/').collect();
        
        // {커리큘럼}/{제목}/{파일명} 패턴 검증
        if path_parts.len() == 3 {
            let curriculum = path_parts[0];
            let title = path_parts[1]; 
            let filename = path_parts[2];
            
            // 파일명이 제목과 일치하는지 확인 (확장자 제외)
            let filename_without_ext = filename.split('.').next().unwrap_or("");
            if filename_without_ext == title {
                let asset_key = format!("{}_{}", curriculum, title);
                asset_groups.entry(asset_key).or_insert_with(Vec::new).push(file);
            }
        }
    }
    
    // AssetInfo로 변환
    let mut assets = Vec::new();
    
    for (asset_key, files) in asset_groups {
        let parts: Vec<&str> = asset_key.split('_').collect();
        if parts.len() < 2 { continue; }
        
        let curriculum = parts[0].to_string();
        let title = parts[1..].join("_");
        
        let mut covers = Vec::new();
        let mut video_url = None;
        
        for file in &files {
            let path_parts: Vec<&str> = file.key.split('/').collect();
            if path_parts.len() == 3 {
                let filename = path_parts[2];
                
                // 이미지 파일 (.png, .jpg, .jpeg)
                if filename.to_lowercase().ends_with(".png") || 
                   filename.to_lowercase().ends_with(".jpg") || 
                   filename.to_lowercase().ends_with(".jpeg") {
                    covers.push(filename.to_string());
                }
                
                // 비디오 파일 (.mp4, .mov, .avi)
                if filename.to_lowercase().ends_with(".mp4") || 
                   filename.to_lowercase().ends_with(".mov") || 
                   filename.to_lowercase().ends_with(".avi") {
                    video_url = Some(format!("https://r2-api.reengki.com/file?key={}", file.key));
                }
            }
        }
        
        // 최소한 이미지와 비디오가 있는 경우만 포함
        if !covers.is_empty() && video_url.is_some() {
            assets.push(AssetInfo {
                id: asset_key,
                book_id: curriculum.clone(),
                title: title.clone(),
                category: None, // 카테고리는 별도로 관리하지 않음
                covers,
                subtitles: vec![], // 자막은 별도 API로 관리
                youtube_links: vec![], // YouTube 링크는 별도로 관리
                video_url,
            });
        }
    }
    
    Ok(assets)
}

pub async fn list_assets(State(app_state): State<AppState>) -> impl IntoResponse {
    info!("Listing all assets");
    
    match get_assets(&app_state, None).await {
        Ok(assets) => {
            (
                StatusCode::OK,
                Json(AssetListResponse { assets })
            ).into_response()
        }
        Err(error) => {
            error!("Failed to list assets: {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to list assets: {}", error)
                }))
            ).into_response()
        }
    }
}

pub async fn filter_assets(
    State(app_state): State<AppState>,
    Query(params): Query<AssetFilterQuery>
) -> impl IntoResponse {
    info!("Filtering assets with params: {:?}", params);
    
    match get_assets(&app_state, params.book_id.as_deref()).await {
        Ok(assets) => {
            let total_found = assets.len();
            (
                StatusCode::OK,
                Json(AssetFilterResponse { assets, total_found })
            ).into_response()
        }
        Err(error) => {
            error!("Failed to filter assets: {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to filter assets: {}", error)
                }))
            ).into_response()
        }
    }
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