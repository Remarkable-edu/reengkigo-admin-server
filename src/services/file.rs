use crate::dto::file::{
    DeleteFileRequest, DeleteFileResponse, FileListResponse, FileUploadResponse,
};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use axum::body::Bytes;
use reqwest::{multipart, Client};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct FileService {
    client: Arc<Client>,
    base_url: String,
    bucket: String,
}

impl FileService {
    pub fn new(base_url: String, bucket: String) -> Self {
        // Create client with increased timeout for large file uploads
        let client = Client::builder()
            .timeout(Duration::from_secs(600)) // 10 minutes timeout for large files
            .connect_timeout(Duration::from_secs(30)) // 30 seconds connection timeout
            .build()
            .expect("Failed to build HTTP client");
            
        Self {
            client: Arc::new(client),
            base_url,
            bucket,
        }
    }

    pub async fn upload_file(
        &self,
        files: Vec<(String, Bytes)>,
        bucket: Option<&str>,
        base_path: &str,
    ) -> Result<FileUploadResponse> {
        let url = format!("{}/upload", self.base_url);
        let bucket_name = bucket.unwrap_or(&self.bucket);
        
        let file_count = files.len();
        tracing::info!("Starting file upload to {}: {} files", base_path, file_count);
        
        let mut all_uploaded = Vec::new();
        
        // Upload each file individually with complete fullpath
        for (filename, bytes) in files {
            let full_path = format!("{}{}", base_path, filename);
            tracing::info!("Uploading file with fullpath: {}", full_path);
            
            let mut form = multipart::Form::new()
                .text("bucket", bucket_name.to_string())
                .text("fullpath", full_path.clone());

            let part = multipart::Part::bytes(bytes.to_vec()).file_name(filename.clone());
            form = form.part("file", part);
            
            let response = self.client
                .post(&url)
                .multipart(form)
                .send()
                .await
                .map_err(|e| {
                    tracing::error!("Failed to send upload request for {}: {}", filename, e);
                    anyhow::anyhow!("Upload request failed for {}: {}", filename, e)
                })?;

            if response.status().is_success() {
                let mut result = response.json::<FileUploadResponse>().await?;
                
                // Map API fields for backward compatibility
                for uploaded_file in &mut result.uploaded {
                    uploaded_file.filename = uploaded_file.original_file.clone();
                    uploaded_file.url = format!("http://localhost:5001/assets/{}", uploaded_file.file);
                }
                
                all_uploaded.extend(result.uploaded);
                tracing::info!("Successfully uploaded file: {}", filename);
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                tracing::error!("Upload failed for {}: {} - {}", filename, status, error_text);
                anyhow::bail!("Failed to upload file {}: {} - {}", filename, status, error_text);
            }
        }
        
        tracing::info!("Successfully uploaded {} files to {}", all_uploaded.len(), base_path);
        
        Ok(FileUploadResponse {
            uploaded: all_uploaded,
        })
    }

    pub async fn list_files(&self, bucket: Option<&str>) -> Result<FileListResponse> {
        let url = format!("{}/all-file", self.base_url);

        let bucket_name = bucket.unwrap_or(&self.bucket);
        let response = self.client
            .get(&url)
            .query(&[("bucket", bucket_name)])
            .send()
            .await?;

        if response.status().is_success() {
            let result = response.json::<FileListResponse>().await?;
            Ok(result)
        } else {
            anyhow::bail!("Failed to list files: {}", response.status())
        }
    }

    pub async fn delete_file(&self, bucket: Option<&str>, key: &str) -> Result<DeleteFileResponse> {
        let url = format!("{}/delete-file", self.base_url);

        let bucket_name = bucket.unwrap_or(&self.bucket);
        let request = DeleteFileRequest {
            bucket: bucket_name.to_string(),
            key: key.to_string(),
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let result = response.json::<DeleteFileResponse>().await?;
            Ok(result)
        } else {
            anyhow::bail!("Failed to delete file: {}", response.status())
        }
    }
    
    pub async fn unlink_file(&self, key: &str) -> Result<()> {
        let url = "https://r2-api.reengki.com/unlink";
        
        let request = serde_json::json!({
            "key": key
        });

        let response = self.client
            .delete(url)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("Successfully deleted file: {}", key);
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Failed to unlink file: {} - {}", status, error_text)
        }
    }

    pub async fn get_folder_files(&self, bucket: Option<&str>, key: &str) -> Result<R2FolderFilesResponse> {
        let url = format!("{}/folder-files", self.base_url);
        let bucket_name = bucket.unwrap_or(&self.bucket);
        
        let response = self.client
            .get(&url)
            .query(&[("bucket", bucket_name), ("key", key)])
            .send()
            .await?;

        if response.status().is_success() {
            let result = response.json::<R2FolderFilesResponse>().await?;
            Ok(result)
        } else {
            anyhow::bail!("Failed to get folder files: {}", response.status())
        }
    }

    pub async fn get_all_files(&self, _bucket: Option<&str>) -> Result<R2AllFilesResponse> {
        tracing::info!("Fetching all files from R2 Worker API (for root folder structure only)");
        
        // Use R2 Worker API with * to get all files (only for root level folder structure)
        let worker_response = self.get_r2_folder_files("*").await?;
        
        // Convert R2WorkerFolderResponse to R2AllFilesResponse
        let files: Vec<R2FileInfo> = worker_response.into_iter()
            .map(|item| R2FileInfo {
                key: item.key.clone(),
                size: item.value.size,
                last_modified: item.value.modified_date.clone(),
                url: format!("https://r2-api.reengki.com/file?key={}", item.key),
            })
            .collect();
        
        let response = R2AllFilesResponse { files };
        
        tracing::info!("Retrieved {} files from R2 API", response.files.len());
        Ok(response)
    }

    pub async fn get_r2_folder_files(&self, key: &str) -> Result<R2WorkerFolderResponse> {
        // Use the R2 worker API for folder-specific files
        let url = "https://reengki-assets-r2-worker.reengkigo.workers.dev/folder-files";
        
        let response = self.client
            .get(url)
            .query(&[("key", key)])
            .send()
            .await?;

        if response.status().is_success() {
            let result = response.json::<R2WorkerFolderResponse>().await?;
            Ok(result)
        } else {
            anyhow::bail!("Failed to get R2 folder files: {}", response.status())
        }
    }
    
    // 폴더 구조를 위한 경로 기반 폴더 조회 (최적화된 버전)
    pub async fn get_folder_structure(&self, prefix: &str) -> Result<Vec<String>> {
        if prefix.is_empty() {
            // 루트 레벨: 모든 최상위 폴더 조회
            let all_files = self.get_all_files(None).await?;
            
            let mut folders = std::collections::HashSet::new();
            
            for file in all_files.files {
                if let Some(first_slash) = file.key.find('/') {
                    let folder_name = &file.key[..first_slash];
                    if !folder_name.is_empty() {
                        folders.insert(folder_name.to_string());
                    }
                }
            }
            
            let mut result: Vec<String> = folders.into_iter().collect();
            result.sort();
            Ok(result)
        } else {
            // 특정 prefix: 해당 폴더만 조회 (최적화)
            let folder_key = format!("{}/", prefix.trim_end_matches('/'));
            let worker_response = self.get_r2_folder_files(&folder_key).await?;
            
            let mut folders = std::collections::HashSet::new();
            
            for item in worker_response {
                // key에서 prefix 이후 부분 추출
                if let Some(remaining) = item.key.strip_prefix(&folder_key) {
                    if let Some(slash_pos) = remaining.find('/') {
                        let folder_name = &remaining[..slash_pos];
                        if !folder_name.is_empty() {
                            folders.insert(folder_name.to_string());
                        }
                    }
                }
            }
            
            let mut result: Vec<String> = folders.into_iter().collect();
            result.sort();
            Ok(result)
        }
    }

    // 캐시 로직 완전 제거 - 항상 실시간 데이터 사용
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2FileInfo {
    pub key: String,
    pub size: u64,
    pub last_modified: String,
    pub url: String,
}

// folder-files API용 구조체 (기존)
#[derive(Debug, Serialize, Deserialize)]
pub struct R2FolderFileInfo {
    pub key: String,
    pub file: String,
    pub size: u64,
    pub created_at: String,
    pub updated_at: String,
    pub subtitle: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct R2FolderFilesResponse {
    pub files: Vec<R2FolderFileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2AllFilesResponse {
    pub files: Vec<R2FileInfo>,
}

// R2 Worker API용 구조체 (새로운 API)
#[derive(Debug, Serialize, Deserialize)]
pub struct R2WorkerFileValue {
    pub file: String,
    pub original_file: String,
    pub size: u64,
    pub subtitle: Vec<String>,
    #[serde(rename = "modifiedDate")]
    pub modified_date: String,
    #[serde(rename = "createDate")]
    pub create_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct R2WorkerFileItem {
    pub key: String,
    pub value: R2WorkerFileValue,
}

// R2 Worker API는 직접 배열을 반환
pub type R2WorkerFolderResponse = Vec<R2WorkerFileItem>;