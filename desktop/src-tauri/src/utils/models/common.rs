use super::types::ModelCategory;
use crate::error::{AppError, Result};
use crate::platform::{desktop, PlatformCapabilities};
use std::path::PathBuf;

/// Get platform-specific base directory for models
pub fn get_models_base_dir() -> Result<PathBuf> {
    desktop().storage_paths().map(|paths| paths.models)
}

/// Get directory path for a specific model category
pub fn get_category_dir(category: ModelCategory) -> Result<PathBuf> {
    let base_dir = get_models_base_dir()?;
    let category_name = match category {
        ModelCategory::Embedding => "embeddings",
    };
    Ok(base_dir.join(category_name))
}

/// Ensure the models directory exists for a specific category
pub fn ensure_models_dir(category: ModelCategory) -> Result<()> {
    let category_dir = get_category_dir(category)?;
    std::fs::create_dir_all(&category_dir).map_err(|e| AppError::Io(e))?;
    Ok(())
}
