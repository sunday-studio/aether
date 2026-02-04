//! Pull: GET /pull?since=ts, decrypt each change, return Vec<ChangeEnvelope>.

use crate::error::{AppError, Result};
use crate::sync::encryption;
use crate::sync::metadata;
use crate::sync::types::{ChangeEnvelope, PullResponse};
use libsql::Database;
use std::io::Write;

// #region agent log
fn debug_log(location: &str, message: &str, data: &str, hypothesis: &str) {
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open("/Users/casprine/Desktop/vendor/sunday-studio/aether/.cursor/debug.log") {
        let _ = writeln!(f, r#"{{"location":"{}","message":"{}","data":{},"hypothesisId":"{}","timestamp":{}}}"#, location, message, data, hypothesis, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
    }
}
// #endregion

/// Returns (envelopes, timestamp). Caller should set last_sync to timestamp after applying.
pub async fn pull(
    db: &Database,
    key: &[u8; 32],
    base_url: &str,
    device_id: &str,
) -> Result<(Vec<ChangeEnvelope>, i64)> {
    let since = metadata::get_last_sync(db).await?.unwrap_or(0);
    let url = format!(
        "{}/pull?since={}&device_id={}",
        base_url.trim_end_matches('/'),
        since,
        urlencoding::encode(device_id)
    );
    tracing::info!("[SYNC-PULL] Pulling changes from {} (since: {}, device: {})", url, since, device_id);
    // #region agent log
    debug_log("pull.rs:25", "pull_start", &format!(r#"{{"since":{},"url":"{}"}}"#, since, url), "H4");
    // #endregion

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
    // #region agent log
    debug_log("pull.rs:52", "server_response", &format!(r#"{{"changes_count":{},"server_timestamp":{},"has_more":{}}}"#, body.changes.len(), body.timestamp, body.has_more), "H4");
    // #endregion

    let mut out = Vec::with_capacity(body.changes.len());
    let mut decrypted = 0;
    let mut decrypt_failures = 0;
    let mut deserialize_failures = 0;
    for ec in &body.changes {
        match encryption::decrypt(key, &ec.nonce, &ec.ciphertext) {
            Ok(plain) => {
                match serde_json::from_slice::<ChangeEnvelope>(&plain) {
                    Ok(envelope) => {
                        out.push(envelope);
                        decrypted += 1;
                    }
                    Err(e) => {
                        deserialize_failures += 1;
                        tracing::warn!("[SYNC-PULL] Failed to deserialize envelope: {}", e);
                        // #region agent log
                        debug_log("pull.rs:68", "deserialize_failure", &format!(r#"{{"error":"{}"}}"#, e.to_string().replace('"', "'")), "H3");
                        // #endregion
                    }
                }
            }
            Err(e) => {
                decrypt_failures += 1;
                tracing::warn!("[SYNC-PULL] Failed to decrypt change: {}", e);
                // #region agent log
                debug_log("pull.rs:76", "decrypt_failure", &format!(r#"{{"error":"{}"}}"#, e.to_string().replace('"', "'")), "H1");
                // #endregion
            }
        }
    }

    // #region agent log
    debug_log("pull.rs:82", "pull_result", &format!(r#"{{"decrypted":{},"decrypt_failures":{},"deserialize_failures":{},"returning_timestamp":{}}}"#, decrypted, decrypt_failures, deserialize_failures, body.timestamp), "H1,H3,H4");
    // #endregion
    tracing::info!("[SYNC-PULL] Successfully decrypted {} changes", decrypted);
    Ok((out, body.timestamp))
}
