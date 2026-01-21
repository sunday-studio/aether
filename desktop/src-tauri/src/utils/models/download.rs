use crate::error::{AppError, Result};
use std::path::Path;
use futures::StreamExt;

/// Download a file from a URL to a destination path with optional progress tracking
pub async fn download_file(
    url: &str,
    path: &Path,
    progress_callback: Option<Box<dyn Fn(f32) + Send + Sync>>,
) -> Result<()> {
    tracing::info!("Downloading file from {} to {:?}", url, path);
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Io(e))?;
    }
    
    // Download with progress tracking
    let client = reqwest::Client::new();
    let response = client
        .get(url)
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
    let mut file = std::fs::File::create(path)
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
    
    tracing::info!("File downloaded successfully to {:?}", path);
    Ok(())
}
