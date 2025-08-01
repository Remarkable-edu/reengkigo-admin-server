use crate::{
    dto::file::{DeleteFileRequest, FileListQuery},
    services::file::FileService,
};
use axum::{
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use tracing::error;

#[utoipa::path(
    post,
    path = "/upload",
    responses(
        (status = 200, description = "Files uploaded successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "file"
)]
pub async fn upload_file(
    State(file_service): State<Arc<FileService>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut files = Vec::new();
    let mut bucket = String::new();
    let mut full_path = String::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        match field.name().unwrap_or("") {
            "file" => {
                let filename = field.file_name().unwrap_or("unknown").to_string();
                let data = field.bytes().await.unwrap();
                files.push((filename, data));
            }
            "bucket" => bucket = field.text().await.unwrap_or_default(),
            "fullpath" => full_path = field.text().await.unwrap_or_default(),
            _ => {}
        }
    }

    if files.is_empty() || full_path.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Missing required fields: file or fullpath"}))
        ).into_response();
    }

    let bucket_param = if bucket.is_empty() { None } else { Some(bucket.as_str()) };
    match file_service.upload_file(files, bucket_param, &full_path).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => {
            error!("Upload failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Upload failed: {}", e)}))
            ).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/all-file",
    params(
        ("bucket" = String, Query, description = "Bucket name")
    ),
    responses(
        (status = 200, description = "Files listed successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "file"
)]
pub async fn list_files(
    State(file_service): State<Arc<FileService>>,
    Query(params): Query<FileListQuery>,
) -> impl IntoResponse {
    let bucket_param = if params.bucket.is_empty() { None } else { Some(params.bucket.as_str()) };
    match file_service.list_files(bucket_param).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => {
            error!("List files failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("List files failed: {}", e)}))
            ).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/delete-file",
    request_body = DeleteFileRequest,
    responses(
        (status = 200, description = "File deleted successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "file"
)]
pub async fn delete_file(
    State(file_service): State<Arc<FileService>>,
    Json(request): Json<DeleteFileRequest>,
) -> impl IntoResponse {
    let bucket_param = if request.bucket.is_empty() { None } else { Some(request.bucket.as_str()) };
    match file_service.delete_file(bucket_param, &request.key).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => {
            error!("Delete file failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Delete file failed: {}", e)}))
            ).into_response()
        }
    }
}