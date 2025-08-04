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


