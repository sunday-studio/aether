use super::types::ModelCategory;
use crate::error::{AppError, Result};
use std::path::PathBuf;

/// Get platform-specific base directory for models
pub fn get_models_base_dir() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME")
            .map_err(|_| AppError::Internal("HOME environment variable not set".to_string()))?;
        Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("Aether")
            .join("models"))
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME")
            .map_err(|_| AppError::Internal("HOME environment variable not set".to_string()))?;
        Ok(PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("aether")
            .join("models"))
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| AppError::Internal("APPDATA environment variable not set".to_string()))?;
        Ok(PathBuf::from(appdata).join("Aether").join("models"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Ok(PathBuf::from(".").join("models"))
    }
}

/// Get directory path for a specific model category
pub fn get_category_dir(category: ModelCategory) -> Result<PathBuf> {
    let base_dir = get_models_base_dir()?;
    let category_name = match category {
        ModelCategory::Transcription => "transcription",
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
