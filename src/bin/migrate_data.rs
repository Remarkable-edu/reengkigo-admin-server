use anyhow::Result;
use mongodb::{bson::oid::ObjectId, Client, Database};
use serde_json;
use std::fs;
use std::path::Path;
use tokio;

use server_test::{
    models::asset::{Asset, SubtitleEntry, YouTubeLink},
    config::AppConfig,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    println!("Starting data migration from existing asset files to MongoDB...");
    
    // Load configuration
    let config = AppConfig::load()?;
    
    // Connect to MongoDB
    let client = Client::with_uri_str(&config.database.url).await?;
    let database = client.database(&config.database.name);
    
    // Clear existing assets collection
    println!("Clearing existing assets collection...");
    let collection = database.collection::<Asset>("assets");
    collection.drop(None).await?;
    println!("✓ Cleared existing assets collection");
    
    // Scan asset directory for existing data
    let asset_root = "asset";
    if !Path::new(asset_root).exists() {
        println!("❌ Asset directory not found: {}", asset_root);
        return Ok(());
    }
    
    // Scan through asset directories
    let mut migrated_count = 0;
    for curriculum_entry in fs::read_dir(asset_root)? {
        let curriculum_entry = curriculum_entry?;
        if !curriculum_entry.file_type()?.is_dir() {
            continue;
        }
        
        let curriculum_name = curriculum_entry.file_name().to_string_lossy().to_string();
        println!("📁 Processing curriculum: {}", curriculum_name);
        
        for month_entry in fs::read_dir(curriculum_entry.path())? {
            let month_entry = month_entry?;
            if !month_entry.file_type()?.is_dir() {
                continue;
            }
            
            let month_name = month_entry.file_name().to_string_lossy().to_string();
            println!("  📁 Processing month: {}", month_name);
            
            // Try to migrate this curriculum-month combination
            match migrate_asset(&database, &curriculum_name, &month_name).await {
                Ok(()) => {
                    migrated_count += 1;
                    println!("  ✓ Migrated: {} - {}", curriculum_name, month_name);
                }
                Err(e) => {
                    println!("  ❌ Failed to migrate {} - {}: {}", curriculum_name, month_name, e);
                }
            }
        }
    }
    
    println!("\n🎉 Migration completed! Migrated {} assets", migrated_count);
    Ok(())
}

async fn migrate_asset(database: &Database, curriculum: &str, month: &str) -> Result<()> {
    let asset_path = format!("asset/{}/{}", curriculum, month);
    
    // Check if required files exist
    let data_json_path = format!("{}/data.json", asset_path);
    let youtube_links_path = format!("{}/youtube/youtube_links.json", asset_path);
    
    if !Path::new(&data_json_path).exists() {
        return Err(anyhow::anyhow!("data.json not found"));
    }
    
    // Read data.json
    let data_content = fs::read_to_string(&data_json_path)?;
    let data_json: serde_json::Value = serde_json::from_str(&data_content)?;
    
    // Extract covers from data.json
    let covers: Vec<String> = data_json["cover"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect();
    
    // Extract subtitles from data.json
    let subtitles: Vec<SubtitleEntry> = data_json["subtitle"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| {
            let page_num = v["pageNum"].as_u64()? as u32;
            let sentence_num = v["sentenceNum"].as_u64()? as u32;
            let text = v["text"].as_str()?.to_string();
            Some(SubtitleEntry::new(page_num, sentence_num, text))
        })
        .collect();
    
    // Read YouTube links if file exists
    let youtube_links: Vec<YouTubeLink> = if Path::new(&youtube_links_path).exists() {
        let youtube_content = fs::read_to_string(&youtube_links_path)?;
        let youtube_json: serde_json::Value = serde_json::from_str(&youtube_content)?;
        
        youtube_json
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| {
                let thumbnail_file = v["thumbnail_file"].as_str()?.to_string();
                let youtube_url = v["youtube_url"].as_str()?.to_string();
                let title = v["title"].as_str().map(|s| s.to_string());
                Some(YouTubeLink::new(thumbnail_file, youtube_url, title))
            })
            .collect()
    } else {
        vec![]
    };
    
    // Get book_id from project_list.yaml
    let book_id = get_book_id_from_mapping(curriculum, month)?;
    
    // Create Asset model
    let mut asset = Asset::new(
        curriculum.to_string(),
        month.to_string(),
        book_id,
        covers,
        subtitles,
        youtube_links,
    );
    asset.id = Some(ObjectId::new());
    
    // Insert into MongoDB
    let collection = database.collection::<Asset>("assets");
    collection.insert_one(&asset, None).await?;
    
    Ok(())
}

fn get_book_id_from_mapping(curriculum: &str, month: &str) -> Result<String> {
    let project_yaml = std::fs::read_to_string("project_list.yaml")?;
    
    // Simple YAML parsing for our specific format
    let lines: Vec<&str> = project_yaml.lines().collect();
    let mut current_curriculum = None;
    
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        
        if trimmed.ends_with(':') && !trimmed.starts_with(' ') {
            current_curriculum = Some(trimmed.trim_end_matches(':'));
        } else if let Some(curr) = current_curriculum {
            // Normalize curriculum names for comparison (handle underscore variations)
            let normalized_curr = curr.replace("_", "-").to_lowercase();
            let normalized_curriculum = curriculum.replace("_", "-").to_lowercase();
            
            if normalized_curr == normalized_curriculum && trimmed.contains(':') {
                let parts: Vec<&str> = trimmed.split(':').collect();
                if parts.len() == 2 {
                    let month_key = parts[0].trim();
                    let book_id = parts[1].trim();
                    
                    // Convert month name to month_XX format
                    let month_num = month_name_to_number(month);
                    let expected_key = format!("month_{:02}", month_num);
                    
                    if month_key == expected_key {
                        return Ok(book_id.to_string());
                    }
                }
            }
        }
    }
    
    Err(anyhow::anyhow!("Book ID not found for {} - {}", curriculum, month))
}

fn month_name_to_number(month_name: &str) -> u8 {
    match month_name {
        "Jan" | "January" => 1,
        "Feb" | "February" => 2,
        "Mar" | "March" => 3,
        "Apr" | "April" => 4,
        "May" => 5,
        "Jun" | "June" => 6,
        "Jul" | "July" => 7,
        "Aug" | "August" => 8,
        "Sep" | "September" => 9,
        "Oct" | "October" => 10,
        "Nov" | "November" => 11,
        "Dec" | "December" => 12,
        _ => 1,
    }
}