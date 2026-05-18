//! Media blob sync: hash, upload, download, exists. Used when syncing media_items.

use crate::error::{AppError, Result};
use sha2::{Digest, Sha256};
use std::time::Instant;

pub fn media_hash(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

pub async fn upload_media(
    base_url: &str,
    hash: &str,
    encrypted: &[u8],
    device_id: &str,
    device_token: &str,
) -> Result<()> {
    let url = format!("{}/media/{}", base_url.trim_end_matches('/'), hash);
    let client = crate::sync::http_client();
    let started = Instant::now();
    let res = crate::sync::authenticated_request(
        &client,
        reqwest::Method::PUT,
        &url,
        device_id,
        device_token,
    )
    .body(encrypted.to_vec())
    .send()
    .await
    .map_err(|e| AppError::Sync(format!("upload_media: {}", e)))?;
    tracing::info!(
        "[SYNC-TIMING] media_upload_request={}ms status={} hash={}",
        started.elapsed().as_millis(),
        res.status(),
        hash
    );
    if !res.status().is_success() {
        return Err(AppError::Sync(format!("upload_media: {}", res.status())));
    }
    Ok(())
}

pub async fn download_media(
    base_url: &str,
    hash: &str,
    device_id: &str,
    device_token: &str,
) -> Result<Vec<u8>> {
    let url = format!("{}/media/{}", base_url.trim_end_matches('/'), hash);
    let client = crate::sync::http_client();
    let started = Instant::now();
    let res = crate::sync::authenticated_request(
        &client,
        reqwest::Method::GET,
        &url,
        device_id,
        device_token,
    )
    .send()
    .await
    .map_err(|e| AppError::Sync(format!("download_media: {}", e)))?;
    tracing::info!(
        "[SYNC-TIMING] media_download_request={}ms status={} hash={}",
        started.elapsed().as_millis(),
        res.status(),
        hash
    );
    if !res.status().is_success() {
        return Err(AppError::Sync(format!("download_media: {}", res.status())));
    }
    let bytes = res
        .bytes()
        .await
        .map_err(|e| AppError::Sync(format!("download_media body: {}", e)))?;
    Ok(bytes.to_vec())
}

pub async fn download_media_decrypt(
    base_url: &str,
    hash: &str,
    key: &[u8; 32],
    device_id: &str,
    device_token: &str,
) -> Result<Vec<u8>> {
    let started = Instant::now();
    let encrypted = download_media(base_url, hash, device_id, device_token).await?;
    let decrypted = crate::sync::encryption::decrypt_blob(key, &encrypted)?;
    tracing::info!(
        "[SYNC-TIMING] media_download_total={}ms hash={} bytes={}",
        started.elapsed().as_millis(),
        hash,
        decrypted.len()
    );
    Ok(decrypted)
}

pub async fn ensure_media_blob(
    db: &libsql::Database,
    media_id: &str,
    base_url: Option<&str>,
    key: Option<&[u8; 32]>,
    device_id: Option<&str>,
    device_token: Option<&str>,
    media_sync_policy: &str,
) -> Result<()> {
    if media_sync_policy != "on_demand" {
        return Ok(());
    }
    let (Some(url), Some(k), Some(device_id), Some(device_token)) =
        (base_url, key, device_id, device_token)
    else {
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
    let meta: serde_json::Value =
        serde_json::from_str(&metadata_str).unwrap_or(serde_json::json!({}));
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
    let bytes = download_media_decrypt(url, hash, k, device_id, device_token).await?;
    std::fs::write(&full, &bytes).map_err(crate::error::AppError::Io)?;
    Ok(())
}

pub async fn check_media_exists(
    base_url: &str,
    hash: &str,
    device_id: &str,
    device_token: &str,
) -> Result<bool> {
    let url = format!("{}/media/{}", base_url.trim_end_matches('/'), hash);
    let client = crate::sync::http_client();
    let res = crate::sync::authenticated_request(
        &client,
        reqwest::Method::HEAD,
        &url,
        device_id,
        device_token,
    )
    .send()
    .await
    .map_err(|e| AppError::Sync(format!("check_media: {}", e)))?;
    Ok(res.status().is_success())
}
