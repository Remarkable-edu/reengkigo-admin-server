use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FileUploadResponse {
    pub uploaded: Vec<UploadedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UploadedFile {
    pub file: String,
    pub original_file: String,
    pub size: u64,
    pub subtitle: Vec<String>,
    // Legacy fields for backward compatibility
    #[serde(default)]
    pub converted: bool,
    #[serde(default, rename = "filename")]
    pub filename: String,
    #[serde(default, rename = "url")]
    pub url: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FileListQuery {
    pub bucket: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FileListResponse {
    pub files: Vec<FileInfo>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FileInfo {
    pub key: String,
    pub last_modified: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeleteFileRequest {
    pub bucket: String,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeleteFileResponse {
    pub key: String,
    pub result: bool,
}