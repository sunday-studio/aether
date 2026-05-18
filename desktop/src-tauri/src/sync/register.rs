//! Enroll this device with the sync server and receive device auth material.

use crate::error::{AppError, Result};
use crate::sync::types::{RegisterRequest, RegisterResponse};
use std::time::Instant;

pub async fn register_with_server(
    base_url: &str,
    device_id: &str,
    hostname: &str,
    server_seed_phrase: &str,
) -> Result<RegisterResponse> {
    let url = format!("{}/register", base_url.trim_end_matches('/'));
    let body = RegisterRequest {
        device_id: device_id.to_string(),
        hostname: hostname.to_string(),
        server_seed_phrase: server_seed_phrase.to_string(),
    };
    let client = crate::sync::http_client();
    let started = Instant::now();
    let res = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Sync(format!("register request: {}", e)))?;
    tracing::info!(
        "[SYNC-TIMING] register_http={}ms status={}",
        started.elapsed().as_millis(),
        res.status()
    );
    if res.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::Sync("wrong server seed phrase".into()));
    }
    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(AppError::Sync(format!(
            "register failed {}: {}",
            status, text
        )));
    }
    res.json()
        .await
        .map_err(|e| AppError::Sync(format!("register response: {}", e)))
}
