use anyhow::Result;
use mongodb::{bson::{oid::ObjectId}, Collection};
use futures::stream::StreamExt;
use std::fs;
use std::path::Path;
use serde_json;

use crate::{
    dto::asset::{AssetResponse, SubtitleResponse, YouTubeLinkResponse, AssetListResponse, FilterParams, FilteredAssetResponse, CreateAssetRequest, UpdateAssetRequest},
    models::asset::{Asset, SubtitleEntry, YouTubeLink},
    services::database::Database,
};

pub struct AssetService;

impl AssetService {
    /// Create a new asset with file system and database operations
    pub async fn create_asset(db: &Database, request: CreateAssetRequest) -> Result<AssetResponse> {
        let collection: Collection<Asset> = db.database.collection("assets");

        // Check if asset already exists
        let existing_filter = mongodb::bson::doc! {
            "curriculum": &request.curriculum,
            "month": &request.month
        };

        if let Ok(Some(_)) = collection.find_one(existing_filter, None).await {
            return Err(anyhow::anyhow!("Asset for {} - {} already exists", request.curriculum, request.month));
        }

        // Get book_id from project_list.yaml mapping
        let book_id = Self::get_book_id_from_mapping(&request.curriculum, &request.month)?;

        // Move uploaded files to proper locations
        let moved_covers = Self::move_uploaded_files_to_asset_folder(&request.covers, &request.curriculum, &request.month, "cover")?;
        
        // Convert DTOs to model structs
        let subtitles: Vec<SubtitleEntry> = request.subtitles.into_iter().map(|s| {
            SubtitleEntry::new(s.page_num, s.sentence_num, s.text)
        }).collect();

        let youtube_links: Vec<YouTubeLink> = request.youtube_links.into_iter().map(|yt| {
            let moved_thumbnail = Self::move_single_uploaded_file_to_asset_folder(&yt.thumbnail_file, &request.curriculum, &request.month, "thumbnail")
                .unwrap_or(yt.thumbnail_file);
            YouTubeLink::new(moved_thumbnail, yt.youtube_url, yt.title)
        }).collect();

        // Create asset model
        let mut asset = Asset::new(
            request.curriculum.clone(),
            request.month.clone(),
            book_id,
            moved_covers,
            subtitles,
            youtube_links.clone(),
        );
        asset.id = Some(ObjectId::new());

        // Create file system structure
        Self::create_asset_folders(&asset)?;
        Self::write_asset_files(&asset)?;

        // Save to database
        let result = collection.insert_one(&asset, None).await?;
        let asset_id = result.inserted_id.as_object_id().unwrap();

        tracing::info!("Created asset: {} - {}", request.curriculum, request.month);
        Ok(Self::asset_to_response(&asset, asset_id))
    }
    
    /// List all assets from database
    pub async fn list_asset(db: &Database) -> Result<AssetListResponse> {
        let collection: Collection<Asset> = db.database.collection("assets");
        
        let mut assets = Vec::new();
        
        match collection.find(None, None).await {
            Ok(mut cursor) => {
                while let Some(result) = cursor.next().await {
                    match result {
                        Ok(asset) => {
                            if let Some(id) = asset.id {
                                assets.push(Self::asset_to_response(&asset, id));
                            }
                        }
                        Err(e) => {
                            tracing::error!("Error processing asset document: {}", e);
                            continue;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error creating cursor for assets: {}", e);
                return Err(e.into());
            }
        }

        let total_count = assets.len();
        tracing::info!("Retrieved {} assets", total_count);

        Ok(AssetListResponse {
            assets,
            total_count,
        })
    }

    /// Get filtered assets based on curriculum, month, or book_id
    pub async fn get_filtered_assets(db: &Database, filters: FilterParams) -> Result<FilteredAssetResponse> {
        let collection: Collection<Asset> = db.database.collection("assets");
        
        // Build MongoDB filter
        let mut mongo_filter = mongodb::bson::doc! {};
        
        if let Some(ref curriculum) = filters.curriculum {
            mongo_filter.insert("curriculum", curriculum);
        }
        
        if let Some(ref month) = filters.month {
            mongo_filter.insert("month", month);
        }
        
        if let Some(ref book_id) = filters.book_id {
            mongo_filter.insert("book_id", book_id);
        }

        let mut matching_assets = Vec::new();

        match collection.find(mongo_filter, None).await {
            Ok(mut cursor) => {
                while let Some(result) = cursor.next().await {
                    match result {
                        Ok(asset) => {
                            if let Some(id) = asset.id {
                                matching_assets.push(Self::asset_to_response(&asset, id));
                            }
                        }
                        Err(e) => {
                            tracing::error!("Error processing filtered asset: {}", e);
                            continue;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error creating cursor for filtered assets: {}", e);
                return Err(e.into());
            }
        }

        let total_found = matching_assets.len();

        Ok(FilteredAssetResponse {
            curriculum: filters.curriculum,
            month: filters.month,
            assets: matching_assets,
            total_found,
        })
    }

    /// Delete asset with file system cleanup
    pub async fn delete_asset(db: &Database, asset_id: &str) -> Result<bool> {
        let collection: Collection<Asset> = db.database.collection("assets");
        let object_id = ObjectId::parse_str(asset_id)?;

        // Get the asset to extract paths before deleting
        if let Ok(Some(asset)) = collection.find_one(mongodb::bson::doc! {"_id": object_id}, None).await {
            // Delete the asset from database first
            let result = collection.delete_one(mongodb::bson::doc! {"_id": object_id}, None).await?;
            
            if result.deleted_count > 0 {
                // Delete the entire asset folder
                let asset_folder = asset.asset_path();
                if let Err(e) = Self::delete_folder_safe(&asset_folder) {
                    tracing::warn!("Failed to delete asset folder {}: {}", asset_folder, e);
                }
                
                tracing::info!("Deleted asset: {} - {}", asset.curriculum, asset.month);
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    /// Update an existing asset
    pub async fn update_asset(db: &Database, asset_id: &str, request: UpdateAssetRequest) -> Result<AssetResponse> {
        let collection: Collection<Asset> = db.database.collection("assets");
        let object_id = ObjectId::parse_str(asset_id)?;

        // Find existing asset
        let mut asset = collection
            .find_one(mongodb::bson::doc! {"_id": object_id}, None)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Asset not found"))?;

        // Update fields if provided
        if let Some(covers) = request.covers {
            // Check if we have new uploaded files in the uploads folder
            let upload_dir = std::path::Path::new("asset/uploads");
            if upload_dir.exists() {
                let uploaded_files: Vec<_> = std::fs::read_dir(upload_dir)?
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                    .collect();
                
                if !uploaded_files.is_empty() && !asset.covers.is_empty() {
                    // Replace existing cover files with uploaded files, keeping original filenames
                    Self::replace_existing_covers_with_uploads(&asset.curriculum, &asset.month, &asset.covers)?;
                    tracing::info!("Replaced existing cover files with uploaded files");
                    // Keep the original covers array unchanged (same filenames)
                } else {
                    // Normal move operation for new files
                    let moved_covers = Self::move_uploaded_files_to_asset_folder(&covers, &asset.curriculum, &asset.month, "cover")?;
                    asset.covers = moved_covers;
                }
            } else {
                // No uploads folder, just update covers normally
                asset.covers = covers;
            }
        }

        if let Some(subtitles_req) = request.subtitles {
            asset.subtitles = subtitles_req.into_iter().map(|s| {
                SubtitleEntry::new(s.page_num, s.sentence_num, s.text)
            }).collect();
        }

        if let Some(youtube_links_req) = request.youtube_links {
            // Move uploaded thumbnail files to proper asset folder if needed
            let moved_youtube_links = youtube_links_req.into_iter().map(|yt| {
                let moved_thumbnail = Self::move_single_uploaded_file_to_asset_folder(&yt.thumbnail_file, &asset.curriculum, &asset.month, "thumbnail")
                    .unwrap_or(yt.thumbnail_file);
                YouTubeLink::new(moved_thumbnail, yt.youtube_url, yt.title)
            }).collect();
            asset.youtube_links = moved_youtube_links;
        }

        // Update timestamp
        asset.updated_at = Some(mongodb::bson::DateTime::now());

        // Update files
        Self::write_asset_files(&asset)?;

        // Update database
        let update_doc = mongodb::bson::doc! {
            "$set": mongodb::bson::to_bson(&asset)?
        };
        
        collection.update_one(
            mongodb::bson::doc! {"_id": object_id},
            update_doc,
            None
        ).await?;

        tracing::info!("Updated asset: {} - {}", asset.curriculum, asset.month);
        Ok(Self::asset_to_response(&asset, object_id))
    }

    /// Helper: Get book_id from project_list.yaml mapping
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
                if curr.eq_ignore_ascii_case(curriculum) && trimmed.contains(':') {
                    let parts: Vec<&str> = trimmed.split(':').collect();
                    if parts.len() == 2 {
                        let month_key = parts[0].trim();
                        let book_id = parts[1].trim();
                        
                        // Convert month name to month_XX format
                        let month_num = Self::month_name_to_number(month);
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

    /// Helper: Convert month name to number
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

    /// Helper: Create asset folder structure
    fn create_asset_folders(asset: &Asset) -> Result<()> {
        let base_path = asset.asset_path();
        
        // Create directories
        fs::create_dir_all(asset.cover_path())?;
        fs::create_dir_all(asset.subtitle_path())?;
        fs::create_dir_all(asset.thumbnail_path())?;
        fs::create_dir_all(asset.youtube_path())?;
        
        tracing::info!("Created folder structure for: {}", base_path);
        Ok(())
    }

    /// Helper: Write asset files (data.json, subtitle.json, youtube_links.json)
    fn write_asset_files(asset: &Asset) -> Result<()> {
        // Write data.json
        let data_json = serde_json::json!({
            "project": asset.curriculum,
            "month": asset.month,
            "cover": asset.covers,
            "subtitle": asset.subtitles
        });
        
        fs::write(asset.data_json_path(), serde_json::to_string_pretty(&data_json)?)?;

        // Write subtitle.json
        let subtitle_json = serde_json::to_string_pretty(&asset.subtitles)?;
        fs::write(asset.subtitle_json_path(), subtitle_json)?;

        // Write youtube_links.json
        let youtube_json = serde_json::to_string_pretty(&asset.youtube_links)?;
        let youtube_file_path = format!("{}/youtube_links.json", asset.youtube_path());
        fs::write(youtube_file_path, youtube_json)?;

        tracing::info!("Wrote asset files for: {} - {}", asset.curriculum, asset.month);
        Ok(())
    }

    /// Helper: Safely delete a folder
    fn delete_folder_safe(folder_path: &str) -> Result<()> {
        let path = Path::new(folder_path);
        
        if path.exists() && path.is_dir() {
            fs::remove_dir_all(path)?;
            tracing::info!("Deleted folder: {}", folder_path);
        } else {
            tracing::warn!("Folder not found for deletion: {}", folder_path);
        }
        
        Ok(())
    }

    /// Helper: Replace existing cover files with uploaded files, keeping original filenames
    fn replace_existing_covers_with_uploads(
        curriculum: &str,
        month: &str,
        existing_covers: &[String]
    ) -> Result<()> {
        let upload_dir = "asset/uploads";
        let target_dir = format!("asset/{}/{}/cover", curriculum, month);
        
        // Get all uploaded files
        let uploaded_files: Vec<_> = std::fs::read_dir(upload_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .collect();
        
        if uploaded_files.is_empty() {
            return Ok(());
        }
        
        // Replace existing covers with uploaded files (one-to-one mapping)
        for (index, cover_path) in existing_covers.iter().enumerate() {
            if let Some(uploaded_file) = uploaded_files.get(index) {
                let uploaded_file_path = uploaded_file.path();
                
                // Extract the original filename from the cover path (e.g., "cover/1_J1R.png" -> "1_J1R.png")
                let original_filename = cover_path.split('/').last().unwrap_or("");
                let target_path = format!("{}/{}", target_dir, original_filename);
                
                // Copy uploaded file to target location with original filename
                std::fs::copy(&uploaded_file_path, &target_path)?;
                
                // Remove the uploaded file
                std::fs::remove_file(&uploaded_file_path)?;
                
                tracing::info!("Replaced {} with uploaded file, keeping original filename", target_path);
            }
        }
        
        // Clean up any remaining uploaded files
        for uploaded_file in uploaded_files.iter().skip(existing_covers.len()) {
            let _ = std::fs::remove_file(uploaded_file.path());
        }
        
        Ok(())
    }

    /// Helper: Move uploaded files to proper asset folder structure
    fn move_uploaded_files_to_asset_folder(
        file_paths: &[String], 
        curriculum: &str, 
        month: &str, 
        subfolder: &str
    ) -> Result<Vec<String>> {
        let mut moved_paths = Vec::new();
        
        for path in file_paths {
            let moved_path = Self::move_single_uploaded_file_to_asset_folder(path, curriculum, month, subfolder)?;
            moved_paths.push(moved_path);
        }
        
        Ok(moved_paths)
    }

    /// Helper: Move single uploaded file to proper asset folder, maintaining original filename if specified
    fn move_single_uploaded_file_to_asset_folder(
        file_path: &str, 
        curriculum: &str, 
        month: &str, 
        subfolder: &str
    ) -> Result<String> {
        // Check if this is an uploaded file that needs to be moved
        if file_path.starts_with("cover/") || file_path.starts_with("thumbnail/") {
            // Extract filename from path like "cover/filename.jpg" or "thumbnail/filename.jpg"
            let uploaded_filename = file_path.split('/').last().unwrap_or("");
            
            // Check if source file exists in upload folder
            let upload_path = format!("asset/uploads/{}", uploaded_filename);
            
            if std::path::Path::new(&upload_path).exists() {
                let target_dir = format!("asset/{}/{}/{}", curriculum, month, subfolder);
                
                // Create target directory if it doesn't exist
                std::fs::create_dir_all(&target_dir)?;
                
                // Use the original filename as specified in file_path
                let original_filename = file_path.split('/').last().unwrap_or(uploaded_filename);
                let target_path = format!("{}/{}", target_dir, original_filename);
                
                // If target file already exists, remove it first (for overwrite)
                if std::path::Path::new(&target_path).exists() {
                    std::fs::remove_file(&target_path)?;
                    tracing::info!("Removed existing file: {}", target_path);
                }
                
                // Copy file from upload to target location with original filename
                std::fs::copy(&upload_path, &target_path)?;
                
                // Remove the uploaded file
                std::fs::remove_file(&upload_path)?;
                
                tracing::info!("Moved file from {} to {} (keeping original filename)", upload_path, target_path);
                
                // Return the relative path for storage in database
                return Ok(format!("{}/{}", subfolder, original_filename));
            }
        }
        
        // If not an upload file or file doesn't exist, return original path
        Ok(file_path.to_string())
    }

    /// Helper: Convert Asset to AssetResponse
    fn asset_to_response(asset: &Asset, id: ObjectId) -> AssetResponse {
        AssetResponse {
            id: id.to_hex(),
            curriculum: asset.curriculum.clone(),
            month: asset.month.clone(),
            book_id: asset.book_id.clone(),
            covers: asset.covers.clone(),
            subtitles: asset.subtitles.iter().map(|s| SubtitleResponse {
                page_num: s.page_num,
                sentence_num: s.sentence_num,
                text: s.text.clone(),
            }).collect(),
            youtube_links: asset.youtube_links.iter().map(|yt| YouTubeLinkResponse {
                thumbnail_file: yt.thumbnail_file.clone(),
                youtube_url: yt.youtube_url.clone(),
                title: yt.title.clone(),
            }).collect(),
            created_at: asset.created_at.map(|dt| dt.to_string()),
            updated_at: asset.updated_at.map(|dt| dt.to_string()),
        }
    }
}