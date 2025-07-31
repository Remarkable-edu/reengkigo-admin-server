use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SubtitleData {
    pub page_num: i32,
    pub sentence_num: i32,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAssetRequest {
    pub book_id: String,
    pub title: String,
    pub category: Option<String>,
    pub cover_image: String,
    pub video_path: String,
    pub subtitles: Vec<SubtitleData>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAssetResponse {
    pub success: bool,
    pub asset_id: Option<String>,
    pub message: String,
    pub cover_image_url: Option<String>,
    pub video_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetListResponse {
    pub assets: Vec<AssetInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YouTubeLink {
    pub thumbnail_file: String,
    pub youtube_url: String,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetInfo {
    pub id: String,
    pub book_id: String,
    pub title: String,
    pub category: Option<String>,
    pub covers: Vec<String>,
    pub subtitles: Vec<SubtitleData>,
    pub youtube_links: Vec<YouTubeLink>,
    pub video_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetFilterQuery {
    pub book_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetFilterResponse {
    pub assets: Vec<AssetInfo>,
    pub total_found: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAssetRequest {
    pub covers: Option<Vec<String>>,
    pub subtitles: Option<Vec<SubtitleData>>,
    pub youtube_links: Option<Vec<YouTubeLink>>,
}