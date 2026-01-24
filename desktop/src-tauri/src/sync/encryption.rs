//! Sync encryption: Argon2id key derivation, ChaCha20-Poly1305 for transport.

use crate::error::{AppError, Result};
use argon2::Argon2;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::ChaCha20Poly1305;
use rand::RngCore;
use sha2::{Digest, Sha256};

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

/// Derive a 32-byte key from passphrase and salt using Argon2id.
pub fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; KEY_LEN]> {
    let mut out = [0u8; KEY_LEN];
    Argon2::default()
        .hash_password_into(passphrase.as_bytes(), salt, &mut out)
        .map_err(|e| AppError::Sync(format!("Argon2: {}", e)))?;
    Ok(out)
}

/// Encrypt plaintext with ChaCha20-Poly1305. Returns (nonce, ciphertext) both base64.
pub fn encrypt(key: &[u8; KEY_LEN], plaintext: &[u8]) -> Result<(String, String)> {
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| AppError::Sync(format!("ChaCha20: {}", e)))?;
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher
        .encrypt((&nonce).into(), plaintext)
        .map_err(|e| AppError::Sync(format!("encrypt: {}", e)))?;
    Ok((
        BASE64.encode(&nonce),
        BASE64.encode(&ciphertext),
    ))
}

/// Decrypt (nonce_base64, ciphertext_base64) with ChaCha20-Poly1305.
pub fn decrypt(
    key: &[u8; KEY_LEN],
    nonce_base64: &str,
    ciphertext_base64: &str,
) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| AppError::Sync(format!("ChaCha20: {}", e)))?;
    let nonce = BASE64
        .decode(nonce_base64)
        .map_err(|e| AppError::Sync(format!("base64 nonce: {}", e)))?;
    let ciphertext = BASE64
        .decode(ciphertext_base64)
        .map_err(|e| AppError::Sync(format!("base64 ciphertext: {}", e)))?;
    let nonce_arr: [u8; NONCE_LEN] = nonce
        .try_into()
        .map_err(|_| AppError::Sync("invalid nonce length".into()))?;
    let plain = cipher
        .decrypt((&nonce_arr).into(), ciphertext.as_ref())
        .map_err(|e| AppError::Sync(format!("decrypt: {}", e)))?;
    Ok(plain)
}

/// Produce a hash of the key for verification (store as key_check).
pub fn key_check_hash(key: &[u8; KEY_LEN]) -> String {
    hex::encode(Sha256::digest(key))
}

/// Generate a random salt for key derivation.
pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut s = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut s);
    s
}

/// Encrypt plaintext for blob storage. Returns 12-byte nonce + raw ciphertext.
/// Use with PUT /media/{hash}; decrypt_blob on download.
pub fn encrypt_blob(key: &[u8; KEY_LEN], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| AppError::Sync(format!("ChaCha20: {}", e)))?;
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher
        .encrypt((&nonce).into(), plaintext)
        .map_err(|e| AppError::Sync(format!("encrypt blob: {}", e)))?;
    let mut out = nonce.to_vec();
    out.extend(ciphertext);
    Ok(out)
}

/// Decrypt a blob body (12-byte nonce + ciphertext).
pub fn decrypt_blob(key: &[u8; KEY_LEN], body: &[u8]) -> Result<Vec<u8>> {
    if body.len() < NONCE_LEN {
        return Err(AppError::Sync("blob too short".into()));
    }
    let (nonce, ct) = body.split_at(NONCE_LEN);
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| AppError::Sync(format!("ChaCha20: {}", e)))?;
    let nonce_arr: [u8; NONCE_LEN] = nonce
        .try_into()
        .map_err(|_| AppError::Sync("invalid nonce length".into()))?;
    let plain = cipher
        .decrypt((&nonce_arr).into(), ct)
        .map_err(|e| AppError::Sync(format!("decrypt blob: {}", e)))?;
    Ok(plain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let pass = "test pass";
        let salt = generate_salt();
        let key = derive_key(pass, &salt).unwrap();
        let plain = b"hello";
        let (n, c) = encrypt(&key, plain).unwrap();
        let out = decrypt(&key, &n, &c).unwrap();
        assert_eq!(&out[..], plain);
    }

    #[test]
    fn key_check() {
        let key = [1u8; 32];
        let h = key_check_hash(&key);
        assert_eq!(h.len(), 64);
    }
}
