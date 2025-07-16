use anyhow::Result;
use serde_json;
use std::fs;
use std::path::Path;
use tokio;

#[derive(serde::Deserialize)]
struct DataJson {
    project: String,
    month: String,
    thumbnail: Option<Vec<ThumbnailEntry>>,
}

#[derive(serde::Deserialize)]
struct ThumbnailEntry {
    file: String,
    youtube: String,
}

#[derive(serde::Serialize)]
struct YouTubeLink {
    thumbnail_file: String,
    youtube_url: String,
    title: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Processing YouTube links from data.json files...");
    
    // Scan asset directory for data.json files
    let asset_root = "asset";
    if !Path::new(asset_root).exists() {
        println!("❌ Asset directory not found: {}", asset_root);
        return Ok(());
    }
    
    let mut processed_count = 0;
    let mut skipped_count = 0;
    
    // Scan through asset directories
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
            
            // Skip non-month directories (like 'subtitle')
            if month_name == "subtitle" {
                continue;
            }
            
            print!("  📁 Processing month: {} ... ", month_name);
            
            // Try to process this curriculum-month combination
            match process_youtube_links(&curriculum_name, &month_name).await {
                Ok(count) => {
                    if count > 0 {
                        processed_count += 1;
                        println!("✓ Created {} YouTube links", count);
                    } else {
                        skipped_count += 1;
                        println!("⚠️ No thumbnail data found");
                    }
                }
                Err(e) => {
                    skipped_count += 1;
                    println!("❌ Failed: {}", e);
                }
            }
        }
    }
    
    println!("\n🎉 Processing completed!");
    println!("✅ Processed: {} folders", processed_count);
    println!("⚠️ Skipped: {} folders", skipped_count);
    
    Ok(())
}

async fn process_youtube_links(curriculum: &str, month: &str) -> Result<usize> {
    let asset_path = format!("asset/{}/{}", curriculum, month);
    let data_json_path = format!("{}/data.json", asset_path);
    let youtube_folder_path = format!("{}/youtube", asset_path);
    let youtube_links_path = format!("{}/youtube_links.json", youtube_folder_path);
    
    // Check if data.json exists
    if !Path::new(&data_json_path).exists() {
        return Err(anyhow::anyhow!("data.json not found"));
    }
    
    // Check if youtube_links.json already exists
    if Path::new(&youtube_links_path).exists() {
        return Err(anyhow::anyhow!("youtube_links.json already exists"));
    }
    
    // Read and parse data.json
    let data_content = fs::read_to_string(&data_json_path)?;
    let data_json: DataJson = serde_json::from_str(&data_content)?;
    
    // Extract thumbnail data
    let thumbnails = match data_json.thumbnail {
        Some(thumbnails) => thumbnails,
        None => return Ok(0), // No thumbnail data
    };
    
    if thumbnails.is_empty() {
        return Ok(0);
    }
    
    // Convert to YouTube links format
    let youtube_links: Vec<YouTubeLink> = thumbnails.into_iter().map(|thumb| {
        // Generate title from filename
        let title = generate_title_from_filename(&thumb.file, curriculum);
        
        YouTubeLink {
            thumbnail_file: thumb.file,
            youtube_url: thumb.youtube,
            title: Some(title),
        }
    }).collect();
    
    // Create youtube folder if it doesn't exist
    fs::create_dir_all(&youtube_folder_path)?;
    
    // Write youtube_links.json
    let youtube_json = serde_json::to_string_pretty(&youtube_links)?;
    fs::write(&youtube_links_path, youtube_json)?;
    
    Ok(youtube_links.len())
}

fn generate_title_from_filename(filename: &str, curriculum: &str) -> String {
    // Extract base name without extension
    let base_name = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename);
    
    // Remove curriculum prefix and convert to readable title
    let title = if let Some(suffix) = base_name.strip_prefix(&format!("{}_", curriculum.to_uppercase())) {
        suffix
    } else if let Some(suffix) = base_name.strip_prefix(&curriculum.to_uppercase()) {
        if suffix.starts_with('_') {
            &suffix[1..]
        } else {
            suffix
        }
    } else {
        base_name
    };
    
    // Convert underscores to spaces and capitalize words
    title
        .replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}