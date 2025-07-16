use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};

/// New Asset DTOs for curriculum -> month -> files structure

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateAssetRequest {
    pub curriculum: String,
    pub month: String,
    pub covers: Vec<String>,
    pub subtitles: Vec<CreateSubtitleRequest>,
    pub youtube_links: Vec<CreateYouTubeLinkRequest>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateSubtitleRequest {
    pub page_num: u32,
    pub sentence_num: u32,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateYouTubeLinkRequest {
    pub thumbnail_file: String,
    pub youtube_url: String,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateAssetRequest {
    pub covers: Option<Vec<String>>,
    pub subtitles: Option<Vec<CreateSubtitleRequest>>,
    pub youtube_links: Option<Vec<CreateYouTubeLinkRequest>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AssetResponse {
    pub id: String,
    pub curriculum: String,
    pub month: String,
    pub book_id: String,
    pub covers: Vec<String>,
    pub subtitles: Vec<SubtitleResponse>,
    pub youtube_links: Vec<YouTubeLinkResponse>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SubtitleResponse {
    pub page_num: u32,
    pub sentence_num: u32,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct YouTubeLinkResponse {
    pub thumbnail_file: String,
    pub youtube_url: String,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AssetListResponse {
    pub assets: Vec<AssetResponse>,
    pub total_count: usize,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct FilterParams {
    pub curriculum: Option<String>,
    pub month: Option<String>,
    pub book_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FilteredAssetResponse {
    pub curriculum: Option<String>,
    pub month: Option<String>,
    pub assets: Vec<AssetResponse>,
    pub total_found: usize,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}
