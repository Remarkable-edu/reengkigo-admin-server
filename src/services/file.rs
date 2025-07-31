use crate::dto::file::{
    DeleteFileRequest, DeleteFileResponse, FileListResponse, FileUploadResponse,
};
use anyhow::Result;
use axum::body::Bytes;
use reqwest::{multipart, Client};
use std::sync::Arc;

#[derive(Clone)]
pub struct FileService {
    client: Arc<Client>,
    base_url: String,
    bucket: String,
}

impl FileService {
    pub fn new(base_url: String, bucket: String) -> Self {
        Self {
            client: Arc::new(Client::new()),
            base_url,
            bucket,
        }
    }

    pub async fn upload_file(
        &self,
        files: Vec<(String, Bytes)>,
        bucket: Option<&str>,
        full_path: &str,
    ) -> Result<FileUploadResponse> {
        let url = format!("{}/upload", self.base_url);
        let bucket_name = bucket.unwrap_or(&self.bucket);
        
        let mut form = multipart::Form::new()
            .text("bucket", bucket_name.to_string())
            .text("fullpath", full_path.to_string());

        for (filename, bytes) in files {
            let part = multipart::Part::bytes(bytes.to_vec()).file_name(filename);
            form = form.part("file", part);
        }

        let response = self.client
            .post(&url)
            .multipart(form)
            .send()
            .await?;

        if response.status().is_success() {
            let mut result = response.json::<FileUploadResponse>().await?;
            
            // Map API fields for backward compatibility
            for uploaded_file in &mut result.uploaded {
                uploaded_file.filename = uploaded_file.original_file.clone();
                uploaded_file.url = format!("http://localhost:5001/assets/{}", uploaded_file.file);
            }
            
            tracing::info!("Successfully uploaded {} files to {}", result.uploaded.len(), full_path);
            Ok(result)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("Upload failed: {} - {}", status, error_text);
            anyhow::bail!("Failed to upload file: {} - {}", status, error_text)
        }
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
}