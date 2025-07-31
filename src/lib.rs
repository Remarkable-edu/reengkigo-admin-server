pub mod config;
pub mod utils;
pub mod services;
pub mod models;
pub mod handlers;
pub mod dto;
pub mod middleware;

use std::sync::Arc;
use utoipa::OpenApi;
pub use utils::ObservabilityManager;
pub use services::file::FileService;
pub use config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub file_service: Arc<FileService>,
    pub config: Arc<AppConfig>,
    pub observability: Arc<ObservabilityManager>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::file::upload_file,
        handlers::file::list_files,
        handlers::file::delete_file,
    ),
    components(schemas(
        dto::file::FileUploadResponse,
        dto::file::UploadedFile,
        dto::file::FileListQuery,
        dto::file::FileListResponse,
        dto::file::FileInfo,
        dto::file::DeleteFileRequest,
        dto::file::DeleteFileResponse
    )),
    tags(
        (name = "file", description = "File management API")
    )
)]
pub struct ApiDoc;