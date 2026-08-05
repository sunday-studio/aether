//! Pull: GET /pull?cursor=... decrypt each change and return envelopes plus next cursor.

use crate::error::{AppError, Result};
use crate::sync::encryption;
use crate::sync::types::{ChangeEnvelope, PullCursor, PullResponse};
use crate::utils::performance_ledger::{record_error, redact_http_response_body};
use serde_json::json;
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
    .map_err(|error| {
        record_error(
            "sync.pull",
            "Pull request could not reach the sync server",
            json!({ "error": error.to_string() }),
        );
        AppError::Sync(format!("pull request: {}", error))
    })?;
    tracing::info!(
        "[SYNC-TIMING] pull_http={}ms status={}",
        http_started.elapsed().as_millis(),
        res.status()
    );

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        record_error(
            "sync.pull",
            "Sync server rejected a pull request",
            json!({
                "status": status.as_u16(),
                "response": redact_http_response_body(&text),
            }),
        );
        return Err(AppError::Sync(format!("pull failed {}: {}", status, text)));
    }

    let body: PullResponse = res
        .json()
        .await
        .map_err(|e| AppError::Sync(format!("pull json: {}", e)))?;

    let decode_started = Instant::now();
    let out = decode_changes(key, &body.changes)?;
    tracing::info!(
        "[SYNC-TIMING] pull_decode={}ms changes_in={} changes_out={}",
        decode_started.elapsed().as_millis(),
        body.changes.len(),
        out.len()
    );

    Ok((out, body.next_cursor, body.has_more))
}

fn decode_changes(
    key: &[u8; 32],
    changes: &[crate::sync::types::EncryptedChange],
) -> Result<Vec<ChangeEnvelope>> {
    changes
        .iter()
        .map(|change| {
            let plain = encryption::decrypt(key, &change.nonce, &change.ciphertext)
                .map_err(|error| AppError::Sync(format!("pull decrypt: {}", error)))?;
            serde_json::from_slice::<ChangeEnvelope>(&plain)
                .map_err(|error| AppError::Sync(format!("pull envelope: {}", error)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::types::EncryptedChange;

    #[test]
    fn refuses_a_pull_page_with_an_undecodable_change() {
        let error = decode_changes(
            &[0; 32],
            &[EncryptedChange {
                nonce: "not-base64".into(),
                ciphertext: "not-base64".into(),
            }],
        )
        .unwrap_err();

        assert!(error.to_string().contains("pull decrypt"));
    }
}
