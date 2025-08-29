use crate::dto::file::{
    DeleteFileRequest, DeleteFileResponse, FileUploadResponse,
};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use axum::body::Bytes;
use reqwest::{multipart, Client};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Clone)]
pub struct FileService {
    client: Arc<Client>,
    base_url: String,
    bucket: String,
    // 카테고리별 전체 데이터 메모리 캐시
    all_files_cache: Arc<RwLock<HashMap<String, AllFilesCache>>>,
}

#[derive(Debug, Clone)]
struct AllFilesCache {
    data: R2WorkerFolderResponse,
    created_at: Instant,
    ttl: Duration,
}

impl AllFilesCache {
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

impl FileService {
    pub fn new(base_url: String, bucket: String) -> Self {
        // Create client with optimized settings for better performance
        let client = Client::builder()
            .timeout(Duration::from_secs(600)) // 10 minutes timeout for large files
            .connect_timeout(Duration::from_secs(10)) // Reduced connection timeout
            .pool_max_idle_per_host(20) // Increase connection pool size
            .pool_idle_timeout(Duration::from_secs(90)) // Keep connections alive longer
            .tcp_nodelay(true) // Disable Nagle's algorithm for lower latency
            // gzip and brotli are enabled by default in reqwest
            .build()
            .expect("Failed to build HTTP client");
            
        Self {
            client: Arc::new(client),
            base_url,
            bucket,
            all_files_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn upload_file(
        &self,
        files: Vec<(String, Bytes)>,
        bucket: Option<&str>,
        base_path: &str,
        category: Option<&str>,
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
            
            // Add category if provided
            if let Some(cat) = category {
                form = form.text("category", cat.to_string());
            }

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
        
        // 업로드 성공 후 관련 캐시 무효화
        self.invalidate_cache_for_path(base_path).await;
        
        tracing::info!("Successfully uploaded {} files to {} and invalidated cache", all_uploaded.len(), base_path);
        
        Ok(FileUploadResponse {
            uploaded: all_uploaded,
        })
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
            // 삭제 성공 후 관련 캐시 무효화
            let cache_path = if key.contains('/') {
                // "book_id/title/file.ext" -> "book_id/title"로 변환
                let parts: Vec<&str> = key.rsplitn(2, '/').collect();
                if parts.len() == 2 {
                    parts[1].to_string() // 마지막 '/' 이전 부분
                } else {
                    key.to_string()
                }
            } else {
                key.to_string()
            };
            
            self.invalidate_cache_for_path(&cache_path).await;
            
            tracing::info!("Successfully deleted file: {} and invalidated cache", key);
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
        // 메모리 캐시에서 전체 데이터 가져오기
        let all_files_data = self.get_cached_all_files().await?;
        
        // R2WorkerFolderResponse를 R2AllFilesResponse로 변환
        let files: Vec<R2FileInfo> = all_files_data.iter()
            .filter_map(|item| {
                // file 필드가 있는 항목만 포함
                if item.value.file.is_some() {
                    Some(R2FileInfo {
                        key: item.key.clone(),
                        size: item.value.size,
                        last_modified: item.value.modified_date.clone().unwrap_or_default(),
                        url: format!("https://r2-api.reengki.com/file?key={}", item.key),
                    })
                } else {
                    None
                }
            })
            .collect();
        
        Ok(R2AllFilesResponse { files })
    }

    pub async fn get_r2_folder_files(&self, key: &str) -> Result<R2WorkerFolderResponse> {
        self.get_r2_folder_files_with_category(key, "reengkigo").await
    }
    
    pub async fn get_r2_folder_files_with_category(&self, key: &str, category: &str) -> Result<R2WorkerFolderResponse> {
        tracing::info!("Getting folder files for key: '{}' with category: '{}' (from memory)", key, category);
        
        // 메모리에서 전체 데이터 가져오기
        let all_files = self.get_cached_all_files_with_category(category).await?;
        
        // key로 필터링 (key는 보통 "folder/" 형태이므로 prefix로 매칭)
        let prefix = if key == "*" {
            // 전체 데이터 요청
            return Ok(all_files);
        } else {
            key.to_string()
        };
        
        let filtered_files: Vec<R2WorkerFileItem> = all_files.into_iter()
            .filter(|item| item.key.starts_with(&prefix))
            .collect();
        
        tracing::info!("Found {} files for key '{}' with category '{}' from memory", filtered_files.len(), key, category);
        Ok(filtered_files)
    }
    
    // 전체 데이터 로드를 위한 직접 API 호출 (최적화된 병렬 페이지네이션)
    async fn get_r2_folder_files_direct(&self, key: &str) -> Result<R2WorkerFolderResponse> {
        self.get_r2_folder_files_direct_with_category(key, "reengkigo").await
    }
    
    async fn get_r2_folder_files_direct_with_category(&self, key: &str, category: &str) -> Result<R2WorkerFolderResponse> {
        let base_url = "https://assets.reengkigo.com/folder-files";
        let start_time = Instant::now();
        
        tracing::info!("Fetching R2 folder files with key: {} and category: {}", key, category);
        
        // Try simple API call first (no pagination parameters)
        let response = self.client
            .get(base_url)
            .query(&[("key", key), ("category", category)])
            .send()
            .await?;
            
        if !response.status().is_success() {
            tracing::error!("API request failed with status: {}", response.status());
            anyhow::bail!("Failed to get R2 folder files: {}", response.status())
        }
        
        // Try to parse as direct array first
        let response_text = response.text().await?;
        tracing::debug!("API response length: {} chars", response_text.len());
        
        // Try parsing as direct array (Vec<R2WorkerFileItem>)
        if let Ok(items) = serde_json::from_str::<R2WorkerFolderResponse>(&response_text) {
            tracing::info!("Direct array parse successful: {} items in {:?}", 
                items.len(), start_time.elapsed());
            return Ok(items);
        }
        
        // If that fails, try parsing as paginated response
        if let Ok(paginated) = serde_json::from_str::<R2WorkerPaginatedResponse>(&response_text) {
            tracing::info!("Paginated response parse successful: {} items in {:?}", 
                paginated.items.len(), start_time.elapsed());
            return Ok(paginated.items);
        }
        
        tracing::error!("Failed to parse API response as either direct array or paginated response");
        tracing::debug!("Response text preview: {}", &response_text[..response_text.len().min(500)]);
        anyhow::bail!("Failed to parse API response")
    }
    
    // 메모리 캐시에서 전체 데이터 가져오기 (캐시가 없으면 로드)
    async fn get_cached_all_files(&self) -> Result<R2WorkerFolderResponse> {
        self.get_cached_all_files_with_category("reengkigo").await
    }
    
    async fn get_cached_all_files_with_category(&self, category: &str) -> Result<R2WorkerFolderResponse> {
        self.ensure_all_files_loaded_with_category(category).await?;
        
        let cache_read = self.all_files_cache.read().await;
        if let Some(cache) = cache_read.get(category) {
            if !cache.is_expired() {
                tracing::info!("Cache hit for all_files category '{}' (age: {:?})", category, cache.created_at.elapsed());
                return Ok(cache.data.clone());
            }
        }
        
        // 캐시가 만료된 경우 다시 로드
        drop(cache_read);
        self.load_all_files_to_cache_with_category(category).await
    }
    
    // 전체 데이터가 캐시에 로드되어 있는지 확인하고 없으면 로드
    async fn ensure_all_files_loaded(&self) -> Result<()> {
        self.ensure_all_files_loaded_with_category("reengkigo").await
    }
    
    async fn ensure_all_files_loaded_with_category(&self, category: &str) -> Result<()> {
        let cache_read = self.all_files_cache.read().await;
        if let Some(cache) = cache_read.get(category) {
            if !cache.is_expired() {
                // 캐시가 곧 만료될 예정이면 백그라운드에서 미리 갱신
                if cache.created_at.elapsed() > Duration::from_secs(480) { // 8분 경과
                    drop(cache_read);
                    self.refresh_cache_in_background_with_category(category);
                }
                return Ok(());
            }
        }
        drop(cache_read);
        
        self.load_all_files_to_cache_with_category(category).await?;
        Ok(())
    }
    
    // 백그라운드에서 캐시 갱신 (현재 캐시는 유지하면서 새 데이터 로드)
    fn refresh_cache_in_background(&self) {
        self.refresh_cache_in_background_with_category("reengkigo");
    }
    
    fn refresh_cache_in_background_with_category(&self, category: &str) {
        let cache = self.all_files_cache.clone();
        let service = self.clone();
        let category = category.to_string();
        
        tokio::spawn(async move {
            tracing::info!("Starting background cache refresh for category: {}", category);
            match service.get_r2_folder_files_direct_with_category("*", &category).await {
                Ok(new_data) => {
                    let ttl = Duration::from_secs(1800); // 30분 TTL (더 길게)
                    let cache_entry = AllFilesCache {
                        data: new_data.clone(),
                        created_at: Instant::now(),
                        ttl,
                    };
                    
                    let mut cache_write = cache.write().await;
                    cache_write.insert(category.clone(), cache_entry);
                    tracing::info!("Background cache refresh completed with {} files for category: {}", new_data.len(), category);
                }
                Err(e) => {
                    tracing::error!("Failed to refresh cache in background for category {}: {}", category, e);
                }
            }
        });
    }
    
    // 전체 데이터를 캐시에 로드
    async fn load_all_files_to_cache(&self) -> Result<R2WorkerFolderResponse> {
        self.load_all_files_to_cache_with_category("reengkigo").await
    }
    
    async fn load_all_files_to_cache_with_category(&self, category: &str) -> Result<R2WorkerFolderResponse> {
        tracing::info!("Loading all files to cache from R2 Worker API for category: {}", category);
        
        // 이미 로딩 중인지 확인 (중복 로드 방지)
        {
            let cache_read = self.all_files_cache.read().await;
            if let Some(cache) = cache_read.get(category) {
                // 다른 스레드가 방금 로드했을 수 있음
                if cache.created_at.elapsed() < Duration::from_secs(5) {
                    tracing::info!("Cache was just loaded by another thread for category: {}", category);
                    return Ok(cache.data.clone());
                }
            }
        }
        
        // R2 Worker API에서 전체 데이터 가져오기 ("*" 사용)
        let worker_response = self.get_r2_folder_files_direct_with_category("*", category).await?;
        
        // 캐시에 저장
        let ttl = Duration::from_secs(1800); // 30분 TTL로 증가
        let cache_entry = AllFilesCache {
            data: worker_response.clone(),
            created_at: Instant::now(),
            ttl,
        };
        
        {
            let mut cache_write = self.all_files_cache.write().await;
            cache_write.insert(category.to_string(), cache_entry);
        }
        
        tracing::info!("Cached all_files with {} files for category: {} (TTL: {:?})", worker_response.len(), category, ttl);
        Ok(worker_response)
    }
    
    // 폴더 구조를 위한 경로 기반 폴더 조회 (메모리 필터링)
    pub async fn get_folder_structure(&self, prefix: &str) -> Result<Vec<String>> {
        self.get_folder_structure_with_category(prefix, "reengkigo").await
    }
    
    pub async fn get_folder_structure_with_category(&self, prefix: &str, category: &str) -> Result<Vec<String>> {
        tracing::info!("Getting folder structure for prefix: '{}' with category: '{}' (from memory)", prefix, category);
        
        // 메모리에서 전체 데이터 가져오기
        let all_files = self.get_cached_all_files_with_category(category).await?;
        
        let mut folders = std::collections::HashSet::new();
        
        if prefix.is_empty() {
            // 루트 레벨: 첫 번째 '/' 이전 부분들 추출
            for item in &all_files {
                if let Some(first_slash) = item.key.find('/') {
                    let folder_name = &item.key[..first_slash];
                    if !folder_name.is_empty() {
                        folders.insert(folder_name.to_string());
                    }
                }
            }
        } else {
            // 특정 prefix: 해당 prefix 아래 폴더들 추출
            let folder_prefix = format!("{}/", prefix.trim_end_matches('/'));
            
            for item in &all_files {
                if let Some(remaining) = item.key.strip_prefix(&folder_prefix) {
                    if let Some(slash_pos) = remaining.find('/') {
                        let folder_name = &remaining[..slash_pos];
                        if !folder_name.is_empty() {
                            folders.insert(folder_name.to_string());
                        }
                    }
                }
            }
        }
        
        let mut result: Vec<String> = folders.into_iter().collect();
        result.sort();
        
        tracing::info!("Found {} folders for prefix '{}' from memory", result.len(), prefix);
        Ok(result)
    }

    // 캐시 관리 메서드들
    
    // 전체 캐시 초기화 (첫 진입 시)
    pub async fn clear_all_cache(&self) {
        let mut cache_write = self.all_files_cache.write().await;
        cache_write.clear();
        tracing::info!("All files cache cleared");
    }
    
    // 업로드/삭제 시 캐시 무효화 
    pub async fn invalidate_cache_for_path(&self, _path: &str) {
        // 모든 카테고리 캐시 무효화
        let mut cache_write = self.all_files_cache.write().await;
        cache_write.clear();
        tracing::info!("All category caches invalidated due to path change: {}", _path);
    }
    
    // 만료된 캐시 정리 (자동으로 처리됨)
    pub async fn cleanup_expired_cache(&self) {
        let mut cache_write = self.all_files_cache.write().await;
        let expired_categories: Vec<String> = cache_write
            .iter()
            .filter(|(_, cache)| cache.is_expired())
            .map(|(category, _)| category.clone())
            .collect();
        
        for category in expired_categories {
            cache_write.remove(&category);
            tracing::info!("Expired cache cleaned up for category: {}", category);
        }
    }
    
    // 캐시 통계 정보 반환
    pub async fn get_cache_stats(&self) -> (usize, usize) {
        let cache_read = self.all_files_cache.read().await;
        let total = cache_read.len();
        let expired = cache_read
            .values()
            .filter(|cache| cache.is_expired())
            .count();
        (total, expired)
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2WorkerFileValue {
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub original_file: Option<String>,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub subtitle: Vec<String>,
    #[serde(rename = "modifiedDate", default)]
    pub modified_date: Option<String>,
    #[serde(rename = "createDate", default)]
    pub create_date: Option<String>,
    #[serde(rename = "play_link", skip_serializing_if = "Option::is_none")]
    pub play_link: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2WorkerFileItem {
    pub key: String,
    pub index: Option<u64>,
    pub value: R2WorkerFileValue,
}

// R2 Worker API 페이지네이션 응답 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2WorkerPaginatedResponse {
    pub items: Vec<R2WorkerFileItem>,
    pub count: u64,
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
    #[serde(rename = "listComplete", default)]
    pub list_complete: Option<bool>,
}

// R2 Worker API는 직접 배열을 반환
pub type R2WorkerFolderResponse = Vec<R2WorkerFileItem>;