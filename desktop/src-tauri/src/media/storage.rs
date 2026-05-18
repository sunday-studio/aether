use crate::db::repositories::MediaRepository;
use crate::error::{AppError, Result};
use crate::utils::generate_id;
use libsql::Database;
use std::path::PathBuf;
use std::sync::Arc;

/// Get platform-specific media directory path
pub fn get_media_directory() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME")
            .map_err(|_| AppError::Internal("HOME environment variable not set".to_string()))?;
        Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("Aether")
            .join("media"))
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME")
            .map_err(|_| AppError::Internal("HOME environment variable not set".to_string()))?;
        Ok(PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("aether")
            .join("media"))
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| AppError::Internal("APPDATA environment variable not set".to_string()))?;
        Ok(PathBuf::from(appdata).join("Aether").join("media"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        // Fallback to current directory
        Ok(PathBuf::from(".").join("media"))
    }
}

/// Ensure media directory exists
pub fn ensure_media_directory() -> Result<PathBuf> {
    let media_dir = get_media_directory()?;
    std::fs::create_dir_all(&media_dir).map_err(|e| AppError::Io(e))?;
    Ok(media_dir)
}

/// Generate blurhash for an image
fn generate_blurhash(image_data: &[u8]) -> Option<String> {
    match image::load_from_memory(image_data) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();

            // Convert to flat Vec<u8> for blurhash (RGBA format)
            let pixels: Vec<u8> = rgba.pixels().flat_map(|p| p.0.iter().copied()).collect();

            // Generate blurhash with 4x3 components (standard)
            // blurhash::encode returns a String directly
            Some(blurhash::encode(4, 3, width, height, &pixels))
        }
        Err(e) => {
            tracing::warn!("Failed to decode image for blurhash generation: {}", e);
            None
        }
    }
}

/// Get file extension from media type and format
fn get_file_extension(media_type: &str, format: Option<&str>) -> String {
    if media_type == "image" {
        // Try to determine from format or default to jpg
        if let Some(fmt) = format {
            match fmt.to_lowercase().as_str() {
                "png" => "png",
                "jpeg" | "jpg" => "jpg",
                "webp" => "webp",
                "gif" => "gif",
                _ => "jpg",
            }
        } else {
            "jpg"
        }
    } else if media_type == "audio" {
        // Audio format handling
        if let Some(fmt) = format {
            match fmt.to_lowercase().as_str() {
                "webm" => "webm",
                "opus" => "webm",
                "mp3" => "mp3",
                "wav" => "wav",
                "m4a" => "m4a",
                "aac" => "m4a",
                _ => "webm",
            }
        } else {
            "webm"
        }
    } else if media_type == "video" {
        // Video format handling
        if let Some(fmt) = format {
            match fmt.to_lowercase().as_str() {
                "mp4" => "mp4",
                "webm" => "webm",
                "mov" => "mov",
                _ => "mp4",
            }
        } else {
            "mp4"
        }
    } else {
        "bin" // Default binary extension
    }
    .to_string()
}

/// Unified function to save media file to filesystem and database
pub async fn save_media_file(
    database: Arc<Database>,
    entity_type: String,
    entity_id: String,
    media_type: String,
    file_data: Vec<u8>,
    additional_metadata: serde_json::Value,
) -> Result<String> {
    // Ensure media directory exists
    let media_dir = ensure_media_directory()?;

    // Generate media ID and file path
    let media_id = generate_id("media");

    // Get format from metadata if available
    let format = additional_metadata.get("format").and_then(|v| v.as_str());

    let file_extension = get_file_extension(&media_type, format);
    let filename = format!("{}.{}", media_id, file_extension);
    let file_path = media_dir.join(&filename);

    // Save file to filesystem
    std::fs::write(&file_path, &file_data).map_err(|e| AppError::Io(e))?;

    // Create metadata JSON, starting with additional metadata
    let mut metadata = additional_metadata;

    // Add size if not already present
    if !metadata.get("size").is_some() {
        metadata["size"] = serde_json::Value::Number(file_data.len().into());
    }

    // Generate blurhash for images
    if media_type == "image" {
        if let Some(blurhash) = generate_blurhash(&file_data) {
            metadata["blurhash"] = serde_json::Value::String(blurhash);
        }
    }

    // Save to database (relative path from media directory)
    let relative_path = filename; // Just the filename, relative to media directory
    let repo = MediaRepository::new(database);
    repo.create(entity_type, entity_id, media_type, relative_path, metadata)
        .await?;

    Ok(media_id)
}

/// Get file path for a media item (returns full path)
pub async fn get_media_file_path(
    database: Arc<Database>,
    media_id: &str,
) -> Result<Option<PathBuf>> {
    let repo = MediaRepository::new(database);
    let media_item = repo.find_by_id(media_id).await?;

    if let Some(item) = media_item {
        let media_dir = get_media_directory()?;
        Ok(Some(media_dir.join(&item.file_path)))
    } else {
        Ok(None)
    }
}

/// Read media file bytes from filesystem
pub async fn read_media_file(database: Arc<Database>, media_id: &str) -> Result<Vec<u8>> {
    let file_path = get_media_file_path(database, media_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Media item {} not found", media_id)))?;

    std::fs::read(&file_path).map_err(|e| AppError::Io(e))
}

/// Delete media file and database record
pub async fn delete_media(database: Arc<Database>, media_id: &str) -> Result<()> {
    let repo = MediaRepository::new(database.clone());

    // Get file path before deleting from database
    let file_path = get_media_file_path(database.clone(), media_id).await?;

    // Delete from database (CASCADE will delete transcriptions)
    repo.delete(media_id).await?;

    // Delete file from filesystem if it exists
    if let Some(path) = file_path {
        if path.exists() {
            if let Err(e) = std::fs::remove_file(&path) {
                tracing::warn!("Failed to delete media file {:?}: {}", path, e);
                // Don't fail the operation if file deletion fails
            }
        }
    }

    Ok(())
}
