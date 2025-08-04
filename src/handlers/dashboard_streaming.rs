// Alternative implementation using streaming for large files
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use tokio::io::AsyncWriteExt;
use tokio::fs::File;
use tempfile::NamedTempFile;
use std::path::PathBuf;
use tracing::{error, info};
use crate::{
    dto::asset::CreateAssetResponse,
    AppState,
};

// Stream large files to temporary storage instead of loading into memory
pub async fn create_asset_streaming(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let file_service = &app_state.file_service;
    let mut book_id = String::new();
    let mut title = String::new();
    let mut category = String::new();
    let mut temp_files = Vec::new();
    let mut subtitles_json = String::new();

    // Parse multipart data with streaming
    while let Some(mut field) = multipart.next_field().await.unwrap_or(None) {
        match field.name().unwrap_or("") {
            "book_id" => book_id = field.text().await.unwrap_or_default(),
            "title" => title = field.text().await.unwrap_or_default(),
            "category" => category = field.text().await.unwrap_or_default(),
            "subtitles" => subtitles_json = field.text().await.unwrap_or_default(),
            "cover_image" | "video_file" => {
                let filename = field.file_name().unwrap_or("unknown").to_string();
                let field_name = field.name().unwrap_or("unknown").to_string();
                info!("Processing {} file: {} (streaming mode)", field_name, filename);
                
                // Create temporary file
                let temp_file = match NamedTempFile::new() {
                    Ok(f) => f,
                    Err(e) => {
                        error!("Failed to create temp file: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(CreateAssetResponse {
                                success: false,
                                asset_id: None,
                                message: format!("임시 파일 생성 실패: {}", e),
                                cover_image_url: None,
                                video_url: None,
                            })
                        ).into_response();
                    }
                };
                
                let temp_path = temp_file.path().to_path_buf();
                let mut file = match File::create(&temp_path).await {
                    Ok(f) => f,
                    Err(e) => {
                        error!("Failed to open temp file for writing: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(CreateAssetResponse {
                                success: false,
                                asset_id: None,
                                message: format!("파일 쓰기 실패: {}", e),
                                cover_image_url: None,
                                video_url: None,
                            })
                        ).into_response();
                    }
                };
                
                let mut total_size = 0u64;
                
                // Stream file data to disk
                while let Some(chunk) = field.chunk().await.unwrap_or(None) {
                    total_size += chunk.len() as u64;
                    
                    // Check size limit during streaming
                    if total_size > 2 * 1024 * 1024 * 1024 {
                        error!("File too large during streaming: {} ({:.2}GB)", filename, total_size as f64 / (1024.0 * 1024.0 * 1024.0));
                        let _ = tokio::fs::remove_file(&temp_path).await;
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
                    
                    if let Err(e) = file.write_all(&chunk).await {
                        error!("Failed to write chunk to temp file: {}", e);
                        let _ = tokio::fs::remove_file(&temp_path).await;
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(CreateAssetResponse {
                                success: false,
                                asset_id: None,
                                message: format!("파일 쓰기 실패: {}", e),
                                cover_image_url: None,
                                video_url: None,
                            })
                        ).into_response();
                    }
                }
                
                // Flush and close file
                if let Err(e) = file.flush().await {
                    error!("Failed to flush temp file: {}", e);
                    let _ = tokio::fs::remove_file(&temp_path).await;
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(CreateAssetResponse {
                            success: false,
                            asset_id: None,
                            message: format!("파일 저장 실패: {}", e),
                            cover_image_url: None,
                            video_url: None,
                        })
                    ).into_response();
                }
                
                info!("Streamed {} file to temp storage: {} ({:.2}MB)", 
                    field_name, filename, total_size as f64 / (1024.0 * 1024.0));
                
                temp_files.push((filename, temp_path, field_name));
            }
            _ => {}
        }
    }
    
    // Now upload temp files to R2
    // ... rest of the implementation would read temp files and upload them
    
    // Clean up temp files
    for (_, path, _) in &temp_files {
        let _ = tokio::fs::remove_file(path).await;
    }
    
    // Return response
    (
        StatusCode::OK,
        Json(CreateAssetResponse {
            success: true,
            asset_id: Some(format!("{}_{}", book_id, title)),
            message: "스트리밍 업로드 구현이 필요합니다".to_string(),
            cover_image_url: None,
            video_url: None,
        })
    ).into_response()
}