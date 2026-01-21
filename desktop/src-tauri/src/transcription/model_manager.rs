use crate::error::{AppError, Result};
use crate::settings;
use libsql::Database;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Arc;
use futures::StreamExt;

/// Model information
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub size: String,
    pub file_size: u64,
    pub download_url: String,
    pub checksum: Option<String>,
    pub is_downloaded: bool,
}

/// Get platform-specific models directory
pub fn get_models_directory() -> Result<PathBuf> {
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
        Ok(PathBuf::from(appdata)
            .join("Aether")
            .join("models"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Ok(PathBuf::from(".").join("models"))
    }
}

/// List available Whisper models
pub fn list_available_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            name: "Tiny".to_string(),
            size: "tiny".to_string(),
            file_size: 150_000_000, // ~150MB
            download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin".to_string(),
            checksum: None,
            is_downloaded: false,
        },
        ModelInfo {
            name: "Base".to_string(),
            size: "base".to_string(),
            file_size: 290_000_000, // ~290MB
            download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin".to_string(),
            checksum: None,
            is_downloaded: false,
        },
        ModelInfo {
            name: "Small".to_string(),
            size: "small".to_string(),
            file_size: 970_000_000, // ~970MB
            download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin".to_string(),
            checksum: None,
            is_downloaded: false,
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
    
    if !model_path.exists() {
        return Ok(false);
    }
    
    // Check file size (basic verification)
    let metadata = std::fs::metadata(&model_path)
        .map_err(|e| AppError::Io(e))?;
    
    // File should be at least 100MB (very basic check)
    Ok(metadata.len() > 100_000_000)
}

/// Download a Whisper model with progress tracking
pub async fn download_model(
    database: Arc<Database>,
    size: &str,
    progress_callback: Option<Box<dyn Fn(f32) + Send + Sync>>,
) -> Result<PathBuf> {
    let models_dir = get_models_directory()?;
    std::fs::create_dir_all(&models_dir)
        .map_err(|e| AppError::Io(e))?;
    
    let model_path = models_dir.join(format!("ggml-{}.bin", size));
    
    // Get download URL for model size
    let models = list_available_models();
    let model = models.iter()
        .find(|m| m.size == size)
        .ok_or_else(|| AppError::BadRequest(format!("Unknown model size: {}", size)))?;
    
    tracing::info!("Downloading model {} from {}", size, model.download_url);
    
    // Download with progress tracking
    let client = reqwest::Client::new();
    let response = client
        .get(&model.download_url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Download failed: {}", e)))?;
    
    if !response.status().is_success() {
        return Err(AppError::Internal(format!(
            "Download failed with status: {}",
            response.status()
        )));
    }
    
    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded = 0u64;
    let mut file = std::fs::File::create(&model_path)
        .map_err(|e| AppError::Io(e))?;
    
    let mut stream = response.bytes_stream();
    use std::io::Write;
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| AppError::Internal(format!("Stream error: {}", e)))?;
        file.write_all(&chunk)
            .map_err(|e| AppError::Io(e))?;
        
        downloaded += chunk.len() as u64;
        
        // Report progress
        if let Some(ref callback) = progress_callback {
            if total_size > 0 {
                let progress = (downloaded as f32 / total_size as f32) * 100.0;
                callback(progress);
            }
        }
    }
    
    tracing::info!("Model {} downloaded successfully", size);
    
    // Update settings to mark as downloaded
    settings::set_setting(
        database,
        &format!("transcription.local_whisper.downloaded_{}", size),
        "true",
    ).await?;
    
    Ok(model_path)
}

/// Delete a downloaded model
pub fn delete_model(size: &str) -> Result<()> {
    let models_dir = get_models_directory()?;
    let model_path = models_dir.join(format!("ggml-{}.bin", size));
    
    if model_path.exists() {
        std::fs::remove_file(&model_path)
            .map_err(|e| AppError::Io(e))?;
        tracing::info!("Deleted model: {}", size);
    }
    
    Ok(())
}

/// Calculate SHA-256 checksum of a file
pub fn calculate_checksum(file_path: &PathBuf) -> Result<String> {
    use std::io::Read;
    
    let mut file = std::fs::File::open(file_path)
        .map_err(|e| AppError::Io(e))?;
    
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192];
    
    loop {
        let bytes_read = file.read(&mut buffer)
            .map_err(|e| AppError::Io(e))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}
