use crate::error::{AppError, Result};
use crate::utils::models::{
    ensure_models_dir, get_category_dir, ModelCategory, ModelInfo as SharedModelInfo,
};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::path::{Path, PathBuf};

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

    has_fastembed_model_files(&model_path)
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

    if model_path.is_file() {
        tracing::warn!(
            "Removing legacy embedding model file before creating model directory: {:?}",
            model_path
        );
        std::fs::remove_file(&model_path).map_err(AppError::Io)?;
    }

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
        if model_path.is_dir() {
            std::fs::remove_dir_all(&model_path).map_err(AppError::Io)?;
        } else {
            std::fs::remove_file(&model_path).map_err(AppError::Io)?;
        }
        tracing::info!("Deleted embedding model: {}", model_name);
    }

    Ok(())
}

fn has_fastembed_model_files(path: &Path) -> Result<bool> {
    let mut stack = vec![path.to_path_buf()];
    let mut has_onnx = false;
    let mut has_tokenizer = false;

    while let Some(current_path) = stack.pop() {
        for entry in std::fs::read_dir(&current_path).map_err(AppError::Io)? {
            let entry = entry.map_err(AppError::Io)?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                stack.push(entry_path);
                continue;
            }

            let file_name = entry_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            if entry_path
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("onnx")
            {
                has_onnx = true;
            }
            if file_name == "tokenizer.json" {
                has_tokenizer = true;
            }

            if has_onnx && has_tokenizer {
                return Ok(true);
            }
        }
    }

    Ok(false)
}
