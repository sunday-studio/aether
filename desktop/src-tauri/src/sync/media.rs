//! Media blob sync: hash, upload, download, exists. Used when syncing media_items.

use crate::error::{AppError, Result};
use sha2::{Digest, Sha256};

/// Content hash for media: `sha256:{hex}`.
pub fn media_hash(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

/// PUT /media/{hash} with encrypted body.
pub async fn upload_media(base_url: &str, hash: &str, encrypted: &[u8]) -> Result<()> {
    let url = format!("{}/media/{}", base_url.trim_end_matches('/'), hash);
    let res = reqwest::Client::new()
        .put(&url)
        .body(encrypted.to_vec())
        .send()
        .await
        .map_err(|e| AppError::Sync(format!("upload_media: {}", e)))?;
    if !res.status().is_success() {
        return Err(AppError::Sync(format!("upload_media: {}", res.status())));
    }
    Ok(())
}

/// GET /media/{hash} returns encrypted bytes.
pub async fn download_media(base_url: &str, hash: &str) -> Result<Vec<u8>> {
    let url = format!("{}/media/{}", base_url.trim_end_matches('/'), hash);
    let res = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Sync(format!("download_media: {}", e)))?;
    if !res.status().is_success() {
        return Err(AppError::Sync(format!("download_media: {}", res.status())));
    }
    let bytes = res
        .bytes()
        .await
        .map_err(|e| AppError::Sync(format!("download_media body: {}", e)))?;
    Ok(bytes.to_vec())
}

/// GET /media/{hash} and decrypt with sync key. Returns plaintext bytes.
pub async fn download_media_decrypt(
    base_url: &str,
    hash: &str,
    key: &[u8; 32],
) -> Result<Vec<u8>> {
    let encrypted = download_media(base_url, hash).await?;
    crate::sync::encryption::decrypt_blob(key, &encrypted)
}

/// For "on_demand" policy: if the media file is missing, download from sync server.
/// No-op when policy != "on_demand", or url/key are None, or content_hash is missing.
pub async fn ensure_media_blob(
    db: &libsql::Database,
    media_id: &str,
    base_url: Option<&str>,
    key: Option<&[u8; 32]>,
    media_sync_policy: &str,
) -> Result<()> {
    if media_sync_policy != "on_demand" {
        return Ok(());
    }
    let (Some(url), Some(k)) = (base_url, key) else {
        return Ok(());
    };
    let conn = db.connect().map_err(crate::error::AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT file_path, metadata FROM media_items WHERE id = ?1 AND (_deleted = 0 OR _deleted IS NULL)",
            libsql::params![media_id],
        )
        .await
        .map_err(crate::error::AppError::LibSQL)?;
    let row = match rows.next().await.map_err(crate::error::AppError::LibSQL)? {
        Some(r) => r,
        None => return Ok(()),
    };
    let file_path: String = row.get(0).map_err(crate::error::AppError::LibSQL)?;
    let metadata_str: String = row.get(1).map_err(crate::error::AppError::LibSQL)?;
    let meta: serde_json::Value = serde_json::from_str(&metadata_str).unwrap_or(serde_json::json!({}));
    let content_hash = meta.get("content_hash").and_then(|v| v.as_str());
    let Some(hash) = content_hash else {
        return Ok(());
    };
    let media_dir = crate::media::get_media_directory()?;
    let full = media_dir.join(&file_path);
    if full.exists() {
        return Ok(());
    }
    if let Some(p) = full.parent() {
        std::fs::create_dir_all(p).map_err(crate::error::AppError::Io)?;
    }
    let bytes = download_media_decrypt(url, hash, k).await?;
    std::fs::write(&full, &bytes).map_err(crate::error::AppError::Io)?;
    Ok(())
}

/// HEAD /media/{hash}.
pub async fn check_media_exists(base_url: &str, hash: &str) -> Result<bool> {
    let url = format!("{}/media/{}", base_url.trim_end_matches('/'), hash);
    let res = reqwest::Client::new()
        .head(&url)
        .send()
        .await
        .map_err(|e| AppError::Sync(format!("check_media: {}", e)))?;
    Ok(res.status().is_success())
}
