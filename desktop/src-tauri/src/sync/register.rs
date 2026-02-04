//! Register this device with the sync server (required when server uses SERVER_PASSPHRASE).

use crate::error::{AppError, Result};

/// Register device with the server. Call before first pull/push when server expects registration.
pub async fn register_with_server(
    base_url: &str,
    device_id: &str,
    hostname: &str,
    passphrase: &str,
) -> Result<()> {
    let url = format!("{}/register", base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "device_id": device_id,
        "hostname": hostname,
        "passphrase": passphrase,
    });
    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Sync(format!("register request: {}", e)))?;
    if res.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::Sync("wrong passphrase".into()));
    }
    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(AppError::Sync(format!("register failed {}: {}", status, text)));
    }
    tracing::info!("[SYNC] Device registered with server");
    Ok(())
}
