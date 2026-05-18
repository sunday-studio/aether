use crate::error::{AppError, Result};
use crate::settings;
use crate::utils::models::{
    download_file, ensure_models_dir, get_category_dir, verify_file, ModelCategory, ModelInfo,
};
use libsql::Database;
use std::path::PathBuf;
use std::sync::Arc;

/// Get platform-specific models directory for transcription
pub fn get_models_directory() -> Result<PathBuf> {
    get_category_dir(ModelCategory::Transcription)
}

/// List available Whisper models
pub fn list_available_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            name: "Tiny".to_string(),
            size: "tiny".to_string(),
            file_size: 150_000_000, // ~150MB
            download_url: Some(
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
                    .to_string(),
            ),
            checksum: None,
            is_downloaded: false,
            dimensions: None,
            category: ModelCategory::Transcription,
        },
        ModelInfo {
            name: "Base".to_string(),
            size: "base".to_string(),
            file_size: 290_000_000, // ~290MB
            download_url: Some(
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
                    .to_string(),
            ),
            checksum: None,
            is_downloaded: false,
            dimensions: None,
            category: ModelCategory::Transcription,
        },
        ModelInfo {
            name: "Small".to_string(),
            size: "small".to_string(),
            file_size: 970_000_000, // ~970MB
            download_url: Some(
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
                    .to_string(),
            ),
            checksum: None,
            is_downloaded: false,
            dimensions: None,
            category: ModelCategory::Transcription,
        },
    ]
}

/// Check if a model is downloaded
pub fn is_model_downloaded(size: &str) -> Result<bool> {
    let models_dir = get_models_directory()?;
    let model_path = models_dir.join(format!("ggml-{}.bin", size));
    Ok(model_path.exists())
}

/// Verify model file integrity (check if file exists and has correct size)
pub fn verify_model(size: &str) -> Result<bool> {
    let models_dir = get_models_directory()?;
    let model_path = models_dir.join(format!("ggml-{}.bin", size));

    // Get expected size from model list
    let models = list_available_models();
    let model = models.iter().find(|m| m.size == size);

    let expected_size = model.map(|m| m.file_size);
    let checksum = model.and_then(|m| m.checksum.as_deref());

    verify_file(&model_path, expected_size, checksum)
}

/// Download a Whisper model with progress tracking
pub async fn download_model(
    database: Arc<Database>,
    size: &str,
    progress_callback: Option<Box<dyn Fn(f32) + Send + Sync>>,
) -> Result<PathBuf> {
    ensure_models_dir(ModelCategory::Transcription)?;

    let models_dir = get_models_directory()?;
    let model_path = models_dir.join(format!("ggml-{}.bin", size));

    // Get download URL for model size
    let models = list_available_models();
    let model = models.iter().find(|m| m.size == size).ok_or_else(|| {
        crate::error::AppError::BadRequest(format!("Unknown model size: {}", size))
    })?;

    let download_url = model.download_url.as_ref().ok_or_else(|| {
        crate::error::AppError::BadRequest(format!("No download URL for model: {}", size))
    })?;

    tracing::info!("Downloading model {} from {}", size, download_url);

    // Use shared download function
    download_file(download_url, &model_path, progress_callback).await?;

    tracing::info!("Model {} downloaded successfully", size);

    // Update settings to mark as downloaded
    settings::set_setting(
        database,
        &format!("transcription.local_whisper.downloaded_{}", size),
        "true",
    )
    .await?;

    Ok(model_path)
}

/// Delete a downloaded model
pub fn delete_model(size: &str) -> Result<()> {
    let models_dir = get_models_directory()?;
    let model_path = models_dir.join(format!("ggml-{}.bin", size));

    if model_path.exists() {
        std::fs::remove_file(&model_path).map_err(|e| AppError::Io(e))?;
        tracing::info!("Deleted model: {}", size);
    }

    Ok(())
}

/// Calculate SHA-256 checksum of a file
pub fn calculate_checksum(file_path: &PathBuf) -> Result<String> {
    crate::utils::models::calculate_checksum(file_path)
}
