use axum::{
    extract::{Multipart, State, Path},
    http::{StatusCode, HeaderMap, HeaderValue},
    response::{Html, IntoResponse, Json},
};
use tracing::{error, info};
use crate::{
    dto::asset::{CreateAssetResponse, SubtitleData},
    AppState,
};
use serde::{Deserialize, Serialize};

pub async fn dashboard_main() -> Html<&'static str> {
    Html(include_str!("../templates/admin-head/dashboard-main.html"))
}

pub async fn dashboard_asset() -> Html<&'static str> {
    Html(include_str!("../templates/admin-head/dashboard-asset.html"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderItem {
    pub name: String,
    pub path: String,
    pub item_type: String, // "folder" or "file"
    pub size: Option<u64>,
    pub file_type: Option<String>, // "image", "video", "other"
    pub url: Option<String>,
    pub modified_at: Option<String>,
    pub children_count: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderContentsResponse {
    pub current_path: String,
    pub items: Vec<FolderItem>,
    pub breadcrumbs: Vec<BreadcrumbItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BreadcrumbItem {
    pub name: String,
    pub path: String,
}

pub async fn get_folder_contents(
    State(app_state): State<AppState>,
    Path(folder_path): Path<String>
) -> impl IntoResponse {
    info!("Getting folder contents for path: {}", folder_path);
    
    match build_folder_structure(&app_state, &folder_path).await {
        Ok(response) => {
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(error) => {
            error!("Failed to get folder contents: {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to get folder contents: {}", error)
                }))
            ).into_response()
        }
    }
}

pub async fn get_root_folders(State(app_state): State<AppState>) -> impl IntoResponse {
    info!("Getting root folders");
    
    match build_folder_structure(&app_state, "").await {
        Ok(response) => {
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(error) => {
            error!("Failed to get root folders: {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to get root folders: {}", error)
                }))
            ).into_response()
        }
    }
}


pub async fn create_asset(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let file_service = &app_state.file_service;
    let mut book_id = String::new();
    let mut title = String::new();
    let mut files = Vec::new();
    let mut subtitles_json = String::new();

    // Parse multipart data
    while let Some(mut field) = multipart.next_field().await.unwrap_or(None) {
        match field.name().unwrap_or("") {
            "book_id" => book_id = field.text().await.unwrap_or_default(),
            "title" => title = field.text().await.unwrap_or_default(),
            "category" => { let _ = field.text().await.unwrap_or_default(); },
            "subtitles" => subtitles_json = field.text().await.unwrap_or_default(),
            "cover_image" | "video_file" => {
                let filename = field.file_name().unwrap_or("unknown").to_string();
                let field_name = field.name().unwrap_or("unknown").to_string();
                info!("Processing {} file: {}", field_name, filename);
                
                // Stream the file in chunks to handle large files efficiently
                let mut data = Vec::new();
                let mut total_size = 0u64;
                
                // Process field chunks efficiently to avoid memory issues
                while let Ok(Some(chunk)) = field.chunk().await {
                    total_size += chunk.len() as u64;
                    
                    // Check file size limit during streaming (2GB)
                    if total_size > 2 * 1024 * 1024 * 1024 {
                        error!("File too large during streaming: {} ({:.2}GB)", filename, total_size as f64 / (1024.0 * 1024.0 * 1024.0));
                        return (
                            StatusCode::PAYLOAD_TOO_LARGE,
                            Json(CreateAssetResponse {
                                success: false,
                                asset_id: None,
                                message: format!("파일이 너무 큽니다: {:.2}GB (최대 2GB)", total_size as f64 / (1024.0 * 1024.0 * 1024.0)),
                                cover_image_url: None,
                                video_url: None,
                            })
                        ).into_response();
                    }
                    
                    data.extend_from_slice(&chunk);
                }
                
                let file_size_mb = total_size as f64 / (1024.0 * 1024.0);
                info!("Streamed {} file: {} ({:.2}MB)", field_name, filename, file_size_mb);
                
                files.push((filename, data.into()));
            }
            _ => {}
        }
    }

    // Validate required fields (only video file is required now, cover image is optional)
    if book_id.is_empty() || title.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CreateAssetResponse {
                success: false,
                asset_id: None,
                message: "필수 필드 누락: 교재 ID, 제목".to_string(),
                cover_image_url: None,
                video_url: None,
            })
        ).into_response();
    }

    // Rename files and validate types
    let mut renamed_files = Vec::new();
    let mut has_video = false;

    for (original_filename, data) in files {
        let extension = original_filename.rfind('.').map(|i| &original_filename[i..]).unwrap_or("");
        let new_filename = format!("{}{}", title, extension);
        
        let lower = new_filename.to_lowercase();
        if lower.ends_with(".mp4") || lower.ends_with(".mov") || lower.ends_with(".avi") {
            has_video = true;
        }
        
        renamed_files.push((new_filename, data));
    }

    // Add subtitle.json file if subtitles are provided
    if !subtitles_json.is_empty() {
        let subtitle_data: axum::body::Bytes = subtitles_json.as_bytes().to_vec().into();
        renamed_files.push(("subtitle.json".to_string(), subtitle_data));
    }

    if !has_video {
        return (
            StatusCode::BAD_REQUEST,
            Json(CreateAssetResponse {
                success: false,
                asset_id: None,
                message: "비디오 파일이 필요합니다".to_string(),
                cover_image_url: None,
                video_url: None,
            })
        ).into_response();
    }

    let full_path = format!("{}/{}/", book_id, title);
    
    info!("Starting upload to external API: path={}, total_files={}", full_path, renamed_files.len());
    let total_size_mb: f64 = renamed_files.iter().map(|(_, data)| data.len() as f64 / (1024.0 * 1024.0)).sum();
    info!("Total upload size: {:.2}MB", total_size_mb);
    
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


async fn build_folder_structure(app_state: &AppState, target_path: &str) -> Result<FolderContentsResponse, String> {
    // 경로 정규화 (빈 문자열은 루트)
    let normalized_path = if target_path.is_empty() || target_path == "/" {
        ""
    } else {
        target_path.trim_start_matches('/').trim_end_matches('/')
    };
    
    info!("Building folder structure for path: '{}'", normalized_path);
    
    if normalized_path.is_empty() {
        // 루트 레벨: 교재ID (첫 번째 레벨) 추출 - 최적화된 방법 사용
        let folder_names = app_state.file_service.get_folder_structure("").await
            .map_err(|e| format!("Failed to get folder structure: {}", e))?;
        
        let folder_items: Vec<FolderItem> = folder_names.into_iter()
            .map(|folder_name| {
                FolderItem {
                    name: folder_name.clone(),
                    path: folder_name,
                    item_type: "folder".to_string(),
                    size: None,
                    file_type: None,
                    url: None,
                    modified_at: None,
                    children_count: None, // 정확한 개수는 생략하여 성능 향상
                }
            })
            .collect();
        
        // 이미 get_folder_structure에서 정렬됨
        
        let breadcrumbs = build_breadcrumbs("");
        
        Ok(FolderContentsResponse {
            current_path: "".to_string(),
            items: folder_items,
            breadcrumbs,
        })
    } else {
        // 특정 폴더 내부: path 깊이에 따라 다른 처리
        let path_parts: Vec<&str> = normalized_path.split('/').collect();
        
        if path_parts.len() == 1 {
            // 교재ID 레벨: 해당 교재의 모든 제목 폴더 표시 - 최적화된 방법 사용
            let curriculum_id = path_parts[0];
            let folder_names = app_state.file_service.get_folder_structure(curriculum_id).await
                .map_err(|e| format!("Failed to get folder structure: {}", e))?;
            
            let folder_items: Vec<FolderItem> = folder_names.into_iter()
                .map(|folder_name| {
                    FolderItem {
                        name: folder_name.clone(),
                        path: format!("{}/{}", normalized_path, folder_name),
                        item_type: "folder".to_string(),
                        size: None,
                        file_type: None,
                        url: None,
                        modified_at: None,
                        children_count: None, // 정확한 개수는 생략하여 성능 향상
                    }
                })
                .collect();
            
            // 이미 get_folder_structure에서 정렬됨
            
            let breadcrumbs = build_breadcrumbs(normalized_path);
            
            Ok(FolderContentsResponse {
                current_path: normalized_path.to_string(),
                items: folder_items,
                breadcrumbs,
            })
        } else {
            // 교재ID/제목 레벨: R2 Worker API 사용하여 파일 목록 가져오기
            let folder_key = format!("{}/", normalized_path); // key는 trailing slash 필요
            
            match app_state.file_service.get_r2_folder_files(&folder_key).await {
                Ok(folder_result) => {
                    let mut file_items: Vec<FolderItem> = folder_result.into_iter()
                        .map(|item| {
                            // item.value.file에서 마지막 '/' 이후의 파일명만 추출
                            let filename = item.value.file
                                .rsplit('/')
                                .next()
                                .unwrap_or(&item.value.file)
                                .to_string();
                            let file_type = get_file_type(&filename);
                            
                            FolderItem {
                                name: filename,
                                path: item.key.clone(),
                                item_type: "file".to_string(),
                                size: Some(item.value.size),
                                file_type: Some(file_type),
                                url: Some(format!("https://r2-api.reengki.com/file?key={}", item.key)),
                                modified_at: Some(item.value.modified_date),
                                children_count: None,
                            }
                        })
                        .collect();
                    
                    file_items.sort_by(|a, b| a.name.cmp(&b.name));
                    
                    let breadcrumbs = build_breadcrumbs(normalized_path);
                    
                    Ok(FolderContentsResponse {
                        current_path: normalized_path.to_string(),
                        items: file_items,
                        breadcrumbs,
                    })
                }
                Err(e) => {
                    error!("Failed to get folder files from R2 Worker API: {}", e);
                    // Fallback: 빈 폴더 반환
                    Ok(FolderContentsResponse {
                        current_path: normalized_path.to_string(),
                        items: vec![],
                        breadcrumbs: build_breadcrumbs(normalized_path),
                    })
                }
            }
        }
    }
}

fn get_file_type(filename: &str) -> String {
    let lower_filename = filename.to_lowercase();
    
    if lower_filename.ends_with(".png") || lower_filename.ends_with(".jpg") || 
       lower_filename.ends_with(".jpeg") || lower_filename.ends_with(".gif") ||
       lower_filename.ends_with(".webp") {
        "image".to_string()
    } else if lower_filename.ends_with(".mp4") || lower_filename.ends_with(".mov") ||
              lower_filename.ends_with(".avi") || lower_filename.ends_with(".mkv") ||
              lower_filename.ends_with(".webm") {
        "video".to_string()
    } else if lower_filename.ends_with(".pdf") {
        "pdf".to_string()
    } else if lower_filename.ends_with(".txt") || lower_filename.ends_with(".json") ||
              lower_filename.ends_with(".xml") || lower_filename.ends_with(".csv") {
        "text".to_string()
    } else {
        "other".to_string()
    }
}

fn build_breadcrumbs(path: &str) -> Vec<BreadcrumbItem> {
    let mut breadcrumbs = vec![
        BreadcrumbItem {
            name: "Home".to_string(),
            path: "".to_string(),
        }
    ];
    
    if !path.is_empty() {
        let parts: Vec<&str> = path.split('/').collect();
        let mut current_path = String::new();
        
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                current_path.push('/');
            }
            current_path.push_str(part);
            
            breadcrumbs.push(BreadcrumbItem {
                name: part.to_string(),
                path: current_path.clone(),
            });
        }
    }
    
    breadcrumbs
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteItemRequest {
    pub key: String,
}

pub async fn delete_item(
    State(app_state): State<AppState>,
    Json(request): Json<DeleteItemRequest>,
) -> impl IntoResponse {
    info!("Deleting item with key: {}", request.key);
    
    match app_state.file_service.unlink_file(&request.key).await {
        Ok(_) => {
            info!("Successfully deleted item: {}", request.key);
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": "Item deleted successfully"
                }))
            ).into_response()
        }
        Err(error) => {
            error!("Failed to delete item {}: {}", request.key, error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to delete item: {}", error)
                }))
            ).into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SubtitleRequest {
    pub book_id: String,
    pub title: String,
}

async fn find_subtitle_filename(book_id: &str, title: &str) -> String {
    let folder_url = format!("https://r2-api.reengki.com/folder-files?key={}/{}", book_id, title);
    
    match reqwest::get(&folder_url).await {
        Ok(folder_response) => {
            if folder_response.status().is_success() {
                match folder_response.text().await {
                    Ok(folder_content) => {
                        if let Ok(folder_data) = serde_json::from_str::<serde_json::Value>(&folder_content) {
                            if let Some(files) = folder_data.as_array() {
                                // Find subtitle file
                                for file in files {
                                    if let Some(key) = file.get("key").and_then(|k| k.as_str()) {
                                        let filename = key.split('/').last().unwrap_or("");
                                        if filename.to_lowercase().ends_with(".json") && 
                                           (filename.to_lowercase().contains("subtitle") || 
                                            filename.to_lowercase().contains("sub")) {
                                            info!("Found subtitle file: {}", filename);
                                            return filename.to_string();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
        }
        Err(_) => {}
    }
    
    // Default fallback
    "subtitle.json".to_string()
}

pub async fn get_subtitle_data(
    State(_app_state): State<AppState>,
    Path((book_id, title)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting subtitle data for: {}/{}", book_id, title);
    
    // Find the actual subtitle filename
    let subtitle_filename = find_subtitle_filename(&book_id, &title).await;
    let subtitle_url = format!("https://r2-api.reengki.com/download/{}/{}/{}", book_id, title, subtitle_filename);
    
    match reqwest::get(&subtitle_url).await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<Vec<SubtitleData>>().await {
                    Ok(subtitle_data) => {
                        info!("Successfully loaded {} subtitle items", subtitle_data.len());
                        (
                            StatusCode::OK,
                            Json(serde_json::json!({
                                "success": true,
                                "data": subtitle_data,
                                "path": format!("{}/{}/{}", book_id, title, subtitle_filename),
                                "filename": subtitle_filename
                            }))
                        ).into_response()
                    }
                    Err(parse_error) => {
                        error!("Failed to parse subtitle JSON: {}", parse_error);
                        (
                            StatusCode::OK,
                            Json(serde_json::json!({
                                "success": true,
                                "data": [],
                                "message": "자막 데이터 파싱 실패"
                            }))
                        ).into_response()
                    }
                }
            } else {
                info!("Subtitle file not found: {} (status: {})", subtitle_url, response.status());
                (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "success": true,
                        "data": [],
                        "message": "자막 파일이 없습니다"
                    }))
                ).into_response()
            }
        }
        Err(error) => {
            error!("Failed to fetch subtitle data: {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("자막 데이터 요청 실패: {}", error)
                }))
            ).into_response()
        }
    }
}

pub async fn get_image_content(
    State(_app_state): State<AppState>,
    Path((book_id, title)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting image content for: {}/{}", book_id, title);
    
    // Find image file first
    match reqwest::get(&format!("https://r2-api.reengki.com/folder-files?key={}/{}", book_id, title)).await {
        Ok(folder_response) => {
            if !folder_response.status().is_success() {
                return (
                    StatusCode::NOT_FOUND,
                    "Folder not found".to_string()
                ).into_response();
            }
            
            match folder_response.text().await {
                Ok(folder_content) => {
                    // Parse folder content to find image file
                    if let Ok(folder_data) = serde_json::from_str::<serde_json::Value>(&folder_content) {
                        if let Some(files) = folder_data.as_array() {
                            // Find image file
                            for file in files {
                                if let Some(key) = file.get("key").and_then(|k| k.as_str()) {
                                    let filename = key.split('/').last().unwrap_or("");
                                    if filename.to_lowercase().ends_with(".jpg") || 
                                       filename.to_lowercase().ends_with(".jpeg") || 
                                       filename.to_lowercase().ends_with(".png") {
                                        
                                        // Get the image extension
                                        let extension = filename.split('.').last().unwrap_or("jpg");
                                        let image_url = format!("https://r2-api.reengki.com/download/{}/{}/{}.{}", book_id, title, title, extension);
                                        
                                        info!("Loading image from: {}", image_url);
                                        
                                        // Fetch the image
                                        match reqwest::get(&image_url).await {
                                            Ok(image_response) => {
                                                if image_response.status().is_success() {
                                                    match image_response.bytes().await {
                                                        Ok(image_bytes) => {
                                                            // Create response with proper content type
                                                            let mut headers = HeaderMap::new();
                                                            let content_type = match extension.to_lowercase().as_str() {
                                                                "png" => "image/png",
                                                                "jpg" | "jpeg" => "image/jpeg",
                                                                _ => "image/jpeg"
                                                            };
                                                            headers.insert("content-type", HeaderValue::from_static(content_type));
                                                            headers.insert("cache-control", HeaderValue::from_static("public, max-age=3600"));
                                                            
                                                            return (StatusCode::OK, headers, image_bytes.to_vec()).into_response();
                                                        }
                                                        Err(e) => {
                                                            error!("Failed to read image bytes: {}", e);
                                                            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read image").into_response();
                                                        }
                                                    }
                                                } else {
                                                    info!("Image not found: {} (status: {})", image_url, image_response.status());
                                                    return (StatusCode::NOT_FOUND, "Image not found").into_response();
                                                }
                                            }
                                            Err(e) => {
                                                error!("Failed to fetch image: {}", e);
                                                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch image").into_response();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    (StatusCode::NOT_FOUND, "No image file found".to_string()).into_response()
                }
                Err(e) => {
                    error!("Failed to read folder response: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read folder data".to_string()).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to fetch folder data: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch folder data".to_string()).into_response()
        }
    }
}

// 캐시 관리 API 엔드포인트들

pub async fn clear_cache(State(app_state): State<AppState>) -> impl IntoResponse {
    info!("Clearing all cache");
    
    app_state.file_service.clear_all_cache().await;
    
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "All cache cleared successfully"
        }))
    ).into_response()
}

pub async fn get_cache_stats(State(app_state): State<AppState>) -> impl IntoResponse {
    info!("Getting cache statistics");
    
    let (total, expired) = app_state.file_service.get_cache_stats().await;
    
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "stats": {
                "total_entries": total,
                "expired_entries": expired,
                "active_entries": total - expired
            }
        }))
    ).into_response()
}

pub async fn cleanup_expired_cache(State(app_state): State<AppState>) -> impl IntoResponse {
    info!("Cleaning up expired cache entries");
    
    app_state.file_service.cleanup_expired_cache().await;
    
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Expired cache entries cleaned up"
        }))
    ).into_response()
}

pub async fn upload_single_file(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    info!("Single file upload request received");
    
    let mut file_data: Option<(String, axum::body::Bytes)> = None;
    let mut full_path = String::new();
    
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        match field.name().unwrap_or("") {
            "file" => {
                let filename = field.file_name().unwrap_or("unknown").to_string();
                let data = match field.bytes().await {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        error!("Failed to read file data: {}", e);
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({
                                "success": false,
                                "error": "Failed to read file data"
                            }))
                        ).into_response();
                    }
                };
                file_data = Some((filename, data));
            }
            "full_path" => {
                full_path = field.text().await.unwrap_or_default();
            }
            _ => {}
        }
    }
    
    if let Some((filename, bytes)) = file_data {
        if full_path.is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Missing full_path parameter"
                }))
            ).into_response();
        }
        
        info!("Uploading file '{}' to full path: '{}'", filename, full_path);
        info!("Full path length: {}, contains slash: {}", full_path.len(), full_path.contains('/'));
        
        // Extract directory path from full_path (remove the filename)
        let base_path = if let Some(last_slash) = full_path.rfind('/') {
            &full_path[..last_slash + 1] // Include the trailing slash
        } else {
            "" // No directory structure, use empty base path
        };
        
        info!("Extracted base_path: '{}' for filename: '{}'", base_path, filename);
        
        // Use the file service to upload the file
        let files = vec![(filename.clone(), bytes)];
        match app_state.file_service.upload_file(files, None, base_path).await {
            Ok(response) => {
                info!("File uploaded successfully to: {}", full_path);
                (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "success": true,
                        "message": "File uploaded successfully",
                        "file_path": full_path,
                        "filename": filename,
                        "details": response
                    }))
                ).into_response()
            }
            Err(e) => {
                error!("Failed to upload file: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("Upload failed: {}", e)
                    }))
                ).into_response()
            }
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": "No file provided"
            }))
        ).into_response()
    }
}

