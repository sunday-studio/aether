use crate::error::{AppError, Result};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

/// Verify file integrity by checking size and optionally checksum
pub fn verify_file(
    path: &Path,
    expected_size: Option<u64>,
    checksum: Option<&str>,
) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    
    let metadata = std::fs::metadata(path)
        .map_err(|e| AppError::Io(e))?;
    
    // Check file size if expected size is provided
    if let Some(expected) = expected_size {
        if metadata.len() != expected {
            return Ok(false);
        }
    }
    
    // Check checksum if provided
    if let Some(expected_checksum) = checksum {
        let calculated = calculate_checksum(path)?;
        if calculated != expected_checksum {
            return Ok(false);
        }
    }
    
    Ok(true)
}

/// Calculate SHA-256 checksum of a file
pub fn calculate_checksum(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path)
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
