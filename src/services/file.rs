use crate::dto::file::{
    DeleteFileRequest, DeleteFileResponse, FileListResponse, FileUploadResponse,
};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use axum::body::Bytes;
use reqwest::{multipart, Client};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct FileService {
    client: Arc<Client>,
    base_url: String,
    bucket: String,
    // 간단한 메모리 캐시 (5분 TTL)
    cache: Arc<RwLock<HashMap<String, (R2AllFilesResponse, Instant)>>>,
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
            cache: Arc::new(RwLock::new(HashMap::new())),
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
        
        // 업로드된 경로의 캐시만 선택적으로 무효화
        // base_path에서 첫 번째 폴더(교재ID)를 추출
        let cache_path = if base_path.contains('/') {
            base_path.split('/').next().unwrap_or("").to_string()
        } else {
            base_path.to_string()
        };
        
        self.invalidate_path_cache(&cache_path).await;
        
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
            // 삭제된 파일 경로의 캐시만 선택적으로 무효화
            let cache_path = if key.contains('/') {
                key.split('/').next().unwrap_or("").to_string()
            } else {
                key.to_string()
            };
            
            self.invalidate_path_cache(&cache_path).await;
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
        let cache_key = "all_files".to_string();
        let cache_ttl = Duration::from_secs(60); // 1분 캐시로 단축
        
        // 캐시에서 확인
        {
            let cache_read = self.cache.read().await;
            if let Some((cached_response, cached_time)) = cache_read.get(&cache_key) {
                if cached_time.elapsed() < cache_ttl {
                    tracing::info!("Returning cached all_files (age: {:?})", cached_time.elapsed());
                    return Ok(cached_response.clone());
                }
            }
        }
        
        tracing::info!("Cache miss or expired, fetching all files from R2 Worker API");
        
        // Use R2 Worker API with * to get all files
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
        
        // 캐시에 저장
        {
            let mut cache_write = self.cache.write().await;
            cache_write.insert(cache_key, (response.clone(), Instant::now()));
        }
        
        tracing::info!("Cached all_files response with {} files", response.files.len());
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
    
    // 폴더 구조 최적화를 위한 경로 기반 폴더 조회
    pub async fn get_folder_structure(&self, prefix: &str) -> Result<Vec<String>> {
        // 먼저 캐시된 전체 파일 목록을 가져옴
        let all_files = self.get_all_files(None).await?;
        
        let mut folders = std::collections::HashSet::new();
        
        for file in all_files.files {
            if file.key.starts_with(prefix) {
                let remaining_path = if prefix.is_empty() {
                    file.key.as_str()
                } else {
                    file.key.strip_prefix(&format!("{}/", prefix.trim_end_matches('/'))).unwrap_or("")
                };
                
                if let Some(first_slash) = remaining_path.find('/') {
                    let folder_name = &remaining_path[..first_slash];
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

    // 전체 캐시 무효화 메서드
    async fn invalidate_cache(&self) {
        let mut cache_write = self.cache.write().await;
        cache_write.clear();
        tracing::info!("All cache invalidated");
    }
    
    // 선택적 캐시 무효화 메서드
    pub async fn invalidate_path_cache(&self, path: &str) {
        let mut cache_write = self.cache.write().await;
        
        // Remove cache entries that match or contain the path
        let keys_to_remove: Vec<String> = cache_write.keys()
            .filter(|key| key.contains(path) || path.is_empty())
            .cloned()
            .collect();
            
        for key in keys_to_remove {
            cache_write.remove(&key);
            tracing::info!("Cache invalidated for key: {}", key);
        }
        
        // Always invalidate the all_files cache when any path changes
        cache_write.remove("all_files");
        tracing::info!("Invalidated cache for path: {}", path);
    }
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