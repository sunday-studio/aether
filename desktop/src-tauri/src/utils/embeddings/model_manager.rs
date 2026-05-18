use crate::error::{AppError, Result};
use crate::utils::models::{
    ensure_models_dir, get_category_dir, ModelCategory, ModelInfo as SharedModelInfo,
};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::path::PathBuf;

/// Model information (re-exported for backward compatibility)
pub type ModelInfo = SharedModelInfo;

/// Get platform-specific model storage path
pub fn get_model_path(model_name: &str) -> Result<PathBuf> {
    let embeddings_dir = get_category_dir(ModelCategory::Embedding)?;
    Ok(embeddings_dir.join(model_name))
}

/// List available embedding models
pub fn list_available_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            name: "all-MiniLM-L6-v2".to_string(),
            size: "small".to_string(),
            dimensions: Some(384),
            file_size: 100_000_000,
            download_url: None,
            checksum: None,
            is_downloaded: false, // Will be checked against filesystem
            category: ModelCategory::Embedding,
        },
        // Can add more models here
    ]
}

/// Check if model is downloaded
pub fn is_model_downloaded(model_name: &str) -> Result<bool> {
    let model_path = get_model_path(model_name)?;
    if !model_path.exists() || !model_path.is_dir() {
        return Ok(false);
    }

    Ok(model_path
        .read_dir()
        .map_err(AppError::Io)?
        .next()
        .transpose()
        .map_err(AppError::Io)?
        .is_some())
}

/// Ensure models directory exists
pub fn ensure_models_directory() -> Result<()> {
    ensure_models_dir(ModelCategory::Embedding)
}

/// Download an embedding model with progress tracking
pub async fn download_model(
    model_name: &str,
    progress_callback: Option<Box<dyn Fn(f32) + Send + Sync>>,
) -> Result<PathBuf> {
    ensure_models_dir(ModelCategory::Embedding)?;

    let model_path = get_model_path(model_name)?;

    let models = list_available_models();
    models
        .iter()
        .find(|m| m.name == model_name)
        .ok_or_else(|| AppError::BadRequest(format!("Unknown model: {}", model_name)))?;

    std::fs::create_dir_all(&model_path).map_err(AppError::Io)?;

    tracing::info!("Preparing local embedding model {}", model_name);

    if let Some(callback) = progress_callback.as_ref() {
        callback(1.0);
    }

    let cache_dir = model_path.clone();
    tokio::task::spawn_blocking(move || {
        let mut options = InitOptions::new(EmbeddingModel::AllMiniLML6V2);
        options.cache_dir = cache_dir;
        options.show_download_progress = false;
        TextEmbedding::try_new(options)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Embedding model setup task failed: {}", e)))?
    .map_err(|e| AppError::Internal(format!("Failed to prepare embedding model: {}", e)))?;

    if let Some(callback) = progress_callback.as_ref() {
        callback(100.0);
    }

    tracing::info!("Embedding model {} downloaded successfully", model_name);

    Ok(model_path)
}

/// Verify embedding model integrity
pub fn verify_model(model_name: &str) -> Result<bool> {
    is_model_downloaded(model_name)
}

/// Delete a downloaded embedding model
pub fn delete_model(model_name: &str) -> Result<()> {
    let model_path = get_model_path(model_name)?;

    if model_path.exists() {
        std::fs::remove_dir_all(&model_path).map_err(AppError::Io)?;
        tracing::info!("Deleted embedding model: {}", model_name);
    }

    Ok(())
}
