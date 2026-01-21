use crate::error::{AppError, Result};
use std::path::{Path, PathBuf};

/// Model information
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub size: String, // "tiny", "base", "small", etc.
    pub dimensions: u32,
    pub file_size: u64,
    pub downloaded: bool,
}

/// Get platform-specific model storage path
pub fn get_model_path(model_name: &str) -> PathBuf {
    let base_dir = get_models_base_dir();
    base_dir.join("embeddings").join(model_name)
}

/// Get platform-specific base directory for models
fn get_models_base_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
        PathBuf::from(home).join("Library/Application Support/Aether/models")
    }
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
        PathBuf::from(home).join(".local/share/aether/models")
    }
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(appdata).join("Aether/models")
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        PathBuf::from("./models")
    }
}

/// List available embedding models
pub fn list_available_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            name: "all-MiniLM-L6-v2".to_string(),
            size: "small".to_string(),
            dimensions: 384,
            file_size: 80_000_000, // ~80MB
            downloaded: false, // Will be checked against filesystem
        },
        // Can add more models here
    ]
}

/// Check if model is downloaded
pub fn is_model_downloaded(model_name: &str) -> bool {
    let model_path = get_model_path(model_name);
    model_path.exists() && model_path.is_file()
}

/// Ensure models directory exists
pub fn ensure_models_dir() -> Result<()> {
    let models_dir = get_models_base_dir().join("embeddings");
    std::fs::create_dir_all(&models_dir)
        .map_err(|e| AppError::Io(e))?;
    Ok(())
}
