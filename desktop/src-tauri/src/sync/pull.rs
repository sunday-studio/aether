//! Pull: GET /pull?cursor=... decrypt each change and return envelopes plus next cursor.

use crate::error::{AppError, Result};
use crate::sync::encryption;
use crate::sync::types::{ChangeEnvelope, PullCursor, PullResponse};
use std::time::Instant;

pub async fn pull(
    key: &[u8; 32],
    base_url: &str,
    device_id: &str,
    device_token: &str,
    cursor: Option<&PullCursor>,
) -> Result<(Vec<ChangeEnvelope>, Option<PullCursor>, bool)> {
    let mut url = format!("{}/pull", base_url.trim_end_matches('/'));
    if let Some(cursor) = cursor {
        url.push_str(&format!(
            "?cursor={}",
            urlencoding::encode(&format!("{}:{}", cursor.received_at, cursor.change_id))
        ));
    }
    tracing::info!(
        "[SYNC-PULL] Pulling changes from {} (device: {})",
        url,
        device_id
    );

    let client = crate::sync::http_client();
    let http_started = Instant::now();
    let res = crate::sync::authenticated_request(
        &client,
        reqwest::Method::GET,
        &url,
        device_id,
        device_token,
    )
    .send()
    .await
    .map_err(|e| AppError::Sync(format!("pull request: {}", e)))?;
    tracing::info!(
        "[SYNC-TIMING] pull_http={}ms status={}",
        http_started.elapsed().as_millis(),
        res.status()
    );

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(AppError::Sync(format!("pull failed {}: {}", status, text)));
    }

    let body: PullResponse = res
        .json()
        .await
        .map_err(|e| AppError::Sync(format!("pull json: {}", e)))?;

    let decode_started = Instant::now();
    let mut out = Vec::with_capacity(body.changes.len());
    for ec in &body.changes {
        match encryption::decrypt(key, &ec.nonce, &ec.ciphertext) {
            Ok(plain) => match serde_json::from_slice::<ChangeEnvelope>(&plain) {
                Ok(envelope) => out.push(envelope),
                Err(e) => tracing::warn!("[SYNC-PULL] Failed to deserialize envelope: {}", e),
            },
            Err(e) => tracing::warn!("[SYNC-PULL] Failed to decrypt change: {}", e),
        }
    }
    tracing::info!(
        "[SYNC-TIMING] pull_decode={}ms changes_in={} changes_out={}",
        decode_started.elapsed().as_millis(),
        body.changes.len(),
        out.len()
    );

    Ok((out, body.next_cursor, body.has_more))
}
