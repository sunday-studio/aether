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
        Ok(PathBuf::from(appdata)
            .join("Aether")
            .join("media"))
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
    std::fs::create_dir_all(&media_dir)
        .map_err(|e| AppError::Io(e))?;
    Ok(media_dir)
}

/// Save audio file to filesystem and metadata to database
pub async fn save_audio_file(
    database: Arc<Database>,
    entry_id: String,
    audio_data: Vec<u8>,
    duration: f32,
    format: String,
) -> Result<String> {
    // Ensure media directory exists
    let media_dir = ensure_media_directory()?;
    
    // Generate media ID and file path
    let media_id = generate_id("media");
    let file_extension = get_file_extension_from_format(&format);
    let filename = format!("{}.{}", media_id, file_extension);
    let file_path = media_dir.join(&filename);
    
    // Save file to filesystem
    std::fs::write(&file_path, &audio_data)
        .map_err(|e| AppError::Io(e))?;
    
    // Create metadata JSON
    let metadata = serde_json::json!({
        "duration": duration,
        "format": format,
        "size": audio_data.len(),
    });
    
    // Save to database (relative path from media directory)
    let relative_path = filename; // Just the filename, relative to media directory
    let repo = MediaRepository::new(database);
    repo.create(
        entry_id,
        "audio".to_string(),
        relative_path,
        metadata,
    ).await?;
    
    Ok(media_id)
}

/// Get file path for audio (returns full path)
pub async fn get_audio_file_path(
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

/// Read audio file bytes from filesystem
pub async fn read_audio_file(
    database: Arc<Database>,
    media_id: &str,
) -> Result<Vec<u8>> {
    let file_path = get_audio_file_path(database, media_id).await?
        .ok_or_else(|| AppError::NotFound(format!("Media item {} not found", media_id)))?;
    
    std::fs::read(&file_path)
        .map_err(|e| AppError::Io(e))
}

/// Delete audio file and database record
pub async fn delete_audio(
    database: Arc<Database>,
    media_id: &str,
) -> Result<()> {
    let repo = MediaRepository::new(database.clone());
    
    // Get file path before deleting from database
    let file_path = get_audio_file_path(database.clone(), media_id).await?;
    
    // Delete from database (CASCADE will delete transcriptions)
    repo.delete(media_id).await?;
    
    // Delete file from filesystem if it exists
    if let Some(path) = file_path {
        if path.exists() {
            if let Err(e) = std::fs::remove_file(&path) {
                tracing::warn!("Failed to delete audio file {:?}: {}", path, e);
                // Don't fail the operation if file deletion fails
            }
        }
    }
    
    Ok(())
}

/// Get file extension from format string
fn get_file_extension_from_format(format: &str) -> &str {
    match format.to_lowercase().as_str() {
        "webm" => "webm",
        "opus" => "webm", // Opus is typically in WebM container
        "mp3" => "mp3",
        "wav" => "wav",
        "m4a" => "m4a",
        "aac" => "m4a",
        _ => "webm", // Default to webm
    }
}
