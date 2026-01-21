use crate::error::{AppError, Result};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use std::sync::Mutex;

// Encryption key storage (in-memory, derived from OS keychain)
static ENCRYPTION_KEY: Mutex<Option<Key<Aes256Gcm>>> = Mutex::new(None);

/// Get or generate encryption key
/// In a production app, this would retrieve from OS keychain
/// For now, we'll generate a key and store it in memory (not secure for production)
fn get_encryption_key() -> Result<Key<Aes256Gcm>> {
    let mut key_guard = ENCRYPTION_KEY.lock().unwrap();
    
    if let Some(key) = *key_guard {
        return Ok(key);
    }
    
    // Generate a new key (in production, retrieve from OS keychain)
    // For now, we'll use a fixed key derived from app identifier
    // TODO: Implement OS keychain integration
    let key_bytes = derive_key_from_app_id()?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    *key_guard = Some(*key);
    
    Ok(*key)
}

/// Derive encryption key from app identifier
/// This is a placeholder - in production, use OS keychain
fn derive_key_from_app_id() -> Result<[u8; 32]> {
    use sha2::{Digest, Sha256};
    
    // Use app identifier as seed
    let app_id = "com.cas.aether";
    let mut hasher = Sha256::new();
    hasher.update(app_id.as_bytes());
    let hash = hasher.finalize();
    
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash[..32]);
    Ok(key)
}

/// Encrypt a value
pub fn encrypt(value: &str) -> Result<String> {
    let key = get_encryption_key()?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    
    let ciphertext = cipher
        .encrypt(&nonce, value.as_bytes())
        .map_err(|e| AppError::Internal(format!("Encryption failed: {}", e)))?;
    
    // Combine nonce and ciphertext, then base64 encode
    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&ciphertext);
    
    Ok(general_purpose::STANDARD.encode(&combined))
}

/// Decrypt a value
pub fn decrypt(encrypted_value: &str) -> Result<String> {
    let key = get_encryption_key()?;
    let cipher = Aes256Gcm::new(&key);
    
    // Decode base64
    let combined = general_purpose::STANDARD
        .decode(encrypted_value)
        .map_err(|e| AppError::Internal(format!("Base64 decode failed: {}", e)))?;
    
    // Extract nonce (first 12 bytes) and ciphertext
    if combined.len() < 12 {
        return Err(AppError::Internal("Invalid encrypted data".to_string()));
    }
    
    let nonce = Nonce::from_slice(&combined[..12]);
    let ciphertext = &combined[12..];
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Internal(format!("Decryption failed: {}", e)))?;
    
    String::from_utf8(plaintext)
        .map_err(|e| AppError::Internal(format!("Invalid UTF-8 in decrypted data: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let original = "test_api_key_12345";
        let encrypted = encrypt(original).unwrap();
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }
}
