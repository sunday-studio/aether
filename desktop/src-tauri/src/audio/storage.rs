use crate::error::Result;
use crate::media::{delete_media, get_media_file_path, read_media_file, save_media_file};
use libsql::Database;
use std::path::PathBuf;
use std::sync::Arc;

// Re-export media directory functions for backward compatibility
pub use crate::media::{ensure_media_directory, get_media_directory};

/// Save audio file to filesystem and metadata to database
pub async fn save_audio_file(
    database: Arc<Database>,
    entry_id: String,
    audio_data: Vec<u8>,
    duration: f32,
    format: String,
) -> Result<String> {
    // Create metadata JSON
    let metadata = serde_json::json!({
        "duration": duration,
        "format": format,
    });

    // Use unified save_media_file function
    save_media_file(
        database,
        "entry".to_string(),
        entry_id,
        "audio".to_string(),
        audio_data,
        metadata,
    )
    .await
}

/// Get file path for audio (returns full path)
pub async fn get_audio_file_path(
    database: Arc<Database>,
    media_id: &str,
) -> Result<Option<PathBuf>> {
    get_media_file_path(database, media_id).await
}

/// Read audio file bytes from filesystem
pub async fn read_audio_file(database: Arc<Database>, media_id: &str) -> Result<Vec<u8>> {
    read_media_file(database, media_id).await
}

/// Delete audio file and database record
pub async fn delete_audio(database: Arc<Database>, media_id: &str) -> Result<()> {
    delete_media(database, media_id).await
}
