//! WebSocket listener: connect to sync server /ws when configured, run sync on "sync" message.

use crate::sync::SyncEngine;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Convert http(s) URL to ws(s) and append /ws with device_id and hostname query params.
fn to_ws_url(base: &str, device_id: &str, hostname: &str) -> String {
    let s = base.trim_end_matches('/');
    let (scheme, rest) = if s.starts_with("https://") {
        ("wss://", &s[8..])
    } else if s.starts_with("http://") {
        ("ws://", &s[7..])
    } else {
        ("wss://", s)
    };
    // URL-encode the hostname since it may contain spaces or special characters
    let encoded_hostname = urlencoding::encode(hostname);
    format!(
        "{}{}/ws?device_id={}&hostname={}",
        scheme, rest, device_id, encoded_hostname
    )
}

/// Run the WebSocket listener. Connects when engine has a URL, reconnects with backoff on disconnect.
/// On each text frame "sync", runs engine.sync() and emits sync-status.
/// Registers with device_id so server can filter out notifications for own pushes.
pub async fn run_ws_listener(engine: Arc<SyncEngine>, app: AppHandle) {
    tracing::info!("[SYNC-WS] Starting WebSocket listener");
    let mut backoff_secs = 1u64;

    loop {
        let url = engine.try_get_url();
        let Some(base_url) = url else {
            tracing::debug!("[SYNC-WS] No server URL configured, waiting 10s");
            tokio::time::sleep(Duration::from_secs(10)).await;
            backoff_secs = 1;
            continue;
        };

        // Get device_id and hostname for registration
        let device_id = match engine.get_device_id().await {
            Ok(id) => id,
            Err(e) => {
                tracing::warn!("[SYNC-WS] Failed to get device_id: {}, retrying in 10s", e);
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        };

        let hostname = match engine.get_device_hostname().await {
            Ok(h) => h,
            Err(e) => {
                tracing::warn!("[SYNC-WS] Failed to get hostname: {}, using 'unknown'", e);
                "unknown".to_string()
            }
        };

        let ws_url = to_ws_url(&base_url, &device_id, &hostname);
        tracing::info!(
            "[SYNC-WS] Connecting to WebSocket: {} (device: {}, hostname: {})",
            ws_url,
            device_id,
            hostname
        );

        match connect_async(&ws_url).await {
            Ok((ws, _)) => {
                tracing::info!("[SYNC-WS] WebSocket connected successfully");
                backoff_secs = 1;

                // Split the websocket for bidirectional communication
                let (mut write, mut read) = ws.split();

                // Clone engine and app for the message handler
                let engine_clone = engine.clone();
                let app_clone = app.clone();

                // Spawn a task to send periodic pings
                let ping_handle = tokio::spawn(async move {
                    let mut interval = tokio::time::interval(Duration::from_secs(30));
                    loop {
                        interval.tick().await;
                        if write.send(Message::Ping(vec![])).await.is_err() {
                            tracing::debug!("[SYNC-WS] Failed to send ping");
                            break;
                        }
                        tracing::debug!("[SYNC-WS] Sent ping");
                    }
                });

                // Read messages from the server
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(t)) => {
                            if t.trim() == "sync" {
                                tracing::info!("[SYNC-WS] Received 'sync' message from server (another device pushed), triggering sync");
                                match engine_clone.sync().await {
                                    Ok(status) => {
                                        tracing::info!(
                                            "[SYNC-WS] Sync triggered by WebSocket completed: pending={}, last_sync={:?}",
                                            status.pending_changes,
                                            status.last_sync
                                        );
                                        let _ = app_clone.emit("sync-status", &status);
                                    }
                                    Err(e) => {
                                        tracing::error!("[SYNC-WS] Sync triggered by WebSocket failed: {}", e);
                                    }
                                }
                            } else {
                                tracing::debug!("[SYNC-WS] Received non-sync message: {}", t);
                            }
                        }
                        Ok(Message::Pong(_)) => {
                            tracing::debug!("[SYNC-WS] Received pong, connection alive");
                        }
                        Ok(Message::Ping(data)) => {
                            tracing::debug!("[SYNC-WS] Received ping from server");
                            // Pong is usually auto-handled by tungstenite, but log it
                            let _ = data; // Acknowledge we received it
                        }
                        Ok(Message::Close(_)) => {
                            tracing::info!("[SYNC-WS] Server closed connection");
                            break;
                        }
                        Err(e) => {
                            tracing::warn!("[SYNC-WS] WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }

                // Clean up ping task
                ping_handle.abort();
                tracing::warn!("[SYNC-WS] WebSocket connection closed");
            }
            Err(e) => {
                tracing::warn!("[SYNC-WS] Failed to connect to WebSocket: {}, retrying in {}s", e, backoff_secs);
            }
        }

        tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
        backoff_secs = (backoff_secs * 2).min(60);
    }
}
