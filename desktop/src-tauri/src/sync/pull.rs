//! Pull: GET /pull?since=ts, decrypt each change, return Vec<ChangeEnvelope>.

use crate::error::{AppError, Result};
use crate::sync::encryption;
use crate::sync::metadata;
use crate::sync::types::{ChangeEnvelope, PullResponse};
use libsql::Database;

/// Returns (envelopes, timestamp). Caller should set last_sync to timestamp after applying.
pub async fn pull(db: &Database, key: &[u8; 32], base_url: &str) -> Result<(Vec<ChangeEnvelope>, i64)> {
    let since = metadata::get_last_sync(db).await?.unwrap_or(0);
    let url = format!("{}/pull?since={}", base_url.trim_end_matches('/'), since);
    tracing::info!("[SYNC-PULL] Pulling changes from {} (since: {})", url, since);

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("[SYNC-PULL] Network error: {}", e);
            AppError::Sync(format!("pull request: {}", e))
        })?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        tracing::error!("[SYNC-PULL] Server returned error {}: {}", status, text);
        return Err(AppError::Sync(format!("pull failed {}: {}", status, text)));
    }

    let body: PullResponse = res
        .json()
        .await
        .map_err(|e| {
            tracing::error!("[SYNC-PULL] JSON parse error: {}", e);
            AppError::Sync(format!("pull json: {}", e))
        })?;

    tracing::info!("[SYNC-PULL] Received {} encrypted changes, timestamp: {}", body.changes.len(), body.timestamp);

    let mut out = Vec::with_capacity(body.changes.len());
    let mut decrypted = 0;
    for ec in &body.changes {
        match encryption::decrypt(key, &ec.nonce, &ec.ciphertext) {
            Ok(plain) => {
                match serde_json::from_slice::<ChangeEnvelope>(&plain) {
                    Ok(envelope) => {
                        out.push(envelope);
                        decrypted += 1;
                    }
                    Err(e) => {
                        tracing::warn!("[SYNC-PULL] Failed to deserialize envelope: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("[SYNC-PULL] Failed to decrypt change: {}", e);
            }
        }
    }

    tracing::info!("[SYNC-PULL] Successfully decrypted {} changes", decrypted);
    Ok((out, body.timestamp))
}
