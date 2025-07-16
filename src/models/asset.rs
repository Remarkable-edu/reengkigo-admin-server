use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Asset document representing educational content assets
/// New structure: curriculum -> month -> { cover, subtitle, thumbnail, youtube_links }
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asset {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub curriculum: String,  // e.g., "jelly", "juice", "stage_1_1"
    pub month: String,       // e.g., "Jan", "Feb", "Mar"
    pub book_id: String,     // from project_list.yaml mapping
    pub covers: Vec<String>, // cover image file paths
    pub subtitles: Vec<SubtitleEntry>, // subtitle data
    pub youtube_links: Vec<YouTubeLink>, // YouTube links with thumbnails
    pub created_at: Option<mongodb::bson::DateTime>,
    pub updated_at: Option<mongodb::bson::DateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubtitleEntry {
    pub page_num: u32,
    pub sentence_num: u32,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YouTubeLink {
    pub thumbnail_file: String, // e.g., "thumbnail/J1R_chant_1.png"
    pub youtube_url: String,    // e.g., "https://youtu.be/tWsBKRctjQg"
    pub title: Option<String>,  // optional title/description
}

impl Asset {
    pub fn new(
        curriculum: String,
        month: String,
        book_id: String,
        covers: Vec<String>,
        subtitles: Vec<SubtitleEntry>,
        youtube_links: Vec<YouTubeLink>,
    ) -> Self {
        let now = mongodb::bson::DateTime::now();
        Self {
            id: None,
            curriculum,
            month,
            book_id,
            covers,
            subtitles,
            youtube_links,
            created_at: Some(now),
            updated_at: Some(now),
        }
    }

    pub fn asset_path(&self) -> String {
        format!("asset/{}/{}", self.curriculum, self.month)
    }

    pub fn cover_path(&self) -> String {
        format!("{}/cover", self.asset_path())
    }

    pub fn subtitle_path(&self) -> String {
        format!("{}/subtitle", self.asset_path())
    }

    pub fn thumbnail_path(&self) -> String {
        format!("{}/thumbnail", self.asset_path())
    }

    pub fn youtube_path(&self) -> String {
        format!("{}/youtube", self.asset_path())
    }

    pub fn data_json_path(&self) -> String {
        format!("{}/data.json", self.asset_path())
    }

    pub fn subtitle_json_path(&self) -> String {
        format!("{}/subtitle.json", self.asset_path())
    }
}

impl SubtitleEntry {
    pub fn new(page_num: u32, sentence_num: u32, text: String) -> Self {
        Self {
            page_num,
            sentence_num,
            text,
        }
    }
}

impl YouTubeLink {
    pub fn new(thumbnail_file: String, youtube_url: String, title: Option<String>) -> Self {
        Self {
            thumbnail_file,
            youtube_url,
            title,
        }
    }
}