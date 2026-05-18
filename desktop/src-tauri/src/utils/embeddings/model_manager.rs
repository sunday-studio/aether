use crate::error::{AppError, Result};
use crate::utils::models::{
    download_file, ensure_models_dir, get_category_dir, verify_file, ModelCategory,
    ModelInfo as SharedModelInfo,
};
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
            file_size: 80_000_000, // ~80MB
            download_url: Some("https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/pytorch_model.bin".to_string()),
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
    Ok(model_path.exists() && model_path.is_file())
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

    // Get download URL for model
    let models = list_available_models();
    let model = models
        .iter()
        .find(|m| m.name == model_name)
        .ok_or_else(|| AppError::BadRequest(format!("Unknown model: {}", model_name)))?;

    let download_url = model.download_url.as_ref().ok_or_else(|| {
        AppError::BadRequest(format!("No download URL for model: {}", model_name))
    })?;

    tracing::info!(
        "Downloading embedding model {} from {}",
        model_name,
        download_url
    );

    // Use shared download function
    download_file(download_url, &model_path, progress_callback).await?;

    tracing::info!("Embedding model {} downloaded successfully", model_name);

    Ok(model_path)
}

/// Verify embedding model integrity
pub fn verify_model(model_name: &str) -> Result<bool> {
    let model_path = get_model_path(model_name)?;

    // Get expected size from model list
    let models = list_available_models();
    let model = models.iter().find(|m| m.name == model_name);

    let expected_size = model.map(|m| m.file_size);
    let checksum = model.and_then(|m| m.checksum.as_deref());

    verify_file(&model_path, expected_size, checksum)
}

/// Delete a downloaded embedding model
pub fn delete_model(model_name: &str) -> Result<()> {
    let model_path = get_model_path(model_name)?;

    if model_path.exists() {
        std::fs::remove_file(&model_path).map_err(|e| AppError::Io(e))?;
        tracing::info!("Deleted embedding model: {}", model_name);
    }

    Ok(())
}
