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

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Sync(format!("pull request: {}", e)))?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(AppError::Sync(format!("pull failed {}: {}", status, text)));
    }

    let body: PullResponse = res
        .json()
        .await
        .map_err(|e| AppError::Sync(format!("pull json: {}", e)))?;

    let mut out = Vec::with_capacity(body.changes.len());
    for ec in &body.changes {
        let plain = encryption::decrypt(key, &ec.nonce, &ec.ciphertext)?;
        let envelope: ChangeEnvelope = serde_json::from_slice(&plain).map_err(AppError::Serialization)?;
        out.push(envelope);
    }

    Ok((out, body.timestamp))
}
