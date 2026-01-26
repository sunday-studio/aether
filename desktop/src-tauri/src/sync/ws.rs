//! WebSocket listener: connect to sync server /ws when configured, run sync on "sync" message.

use crate::sync::SyncEngine;
use futures::StreamExt;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Convert http(s) URL to ws(s) and append /ws.
fn to_ws_url(base: &str) -> String {
    let s = base.trim_end_matches('/');
    let (scheme, rest) = if s.starts_with("https://") {
        ("wss://", &s[8..])
    } else if s.starts_with("http://") {
        ("ws://", &s[7..])
    } else {
        ("wss://", s)
    };
    format!("{}{}/ws", scheme, rest)
}

/// Run the WebSocket listener. Connects when engine has a URL, reconnects with backoff on disconnect.
/// On each text frame "sync", runs engine.sync() and emits sync-status.
pub async fn run_ws_listener(engine: Arc<SyncEngine>, app: AppHandle) {
    tracing::info!("[SYNC-WS] Starting WebSocket listener");
    let mut backoff_secs = 1u64;
    loop {
        let url = engine.try_get_url();
        let Some(ws_url) = url else {
            tracing::debug!("[SYNC-WS] No server URL configured, waiting 10s");
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            backoff_secs = 1;
            continue;
        };
        let ws_url = to_ws_url(&ws_url);
        tracing::info!("[SYNC-WS] Connecting to WebSocket: {}", ws_url);
        match connect_async(&ws_url).await {
            Ok((mut ws, _)) => {
                tracing::info!("[SYNC-WS] WebSocket connected successfully");
                backoff_secs = 1;
                while let Some(msg) = ws.next().await {
                    let Ok(msg) = msg else {
                        tracing::warn!("[SYNC-WS] WebSocket message error, disconnecting");
                        break;
                    };
                    if let Message::Text(t) = msg {
                        if t.trim() == "sync" {
                            tracing::info!("[SYNC-WS] Received 'sync' message from server, triggering sync");
                            match engine.sync().await {
                                Ok(status) => {
                                    tracing::info!("[SYNC-WS] Sync triggered by WebSocket completed: pending={}, last_sync={:?}", 
                                        status.pending_changes, status.last_sync);
                                    let _ = app.emit("sync-status", &status);
                                }
                                Err(e) => {
                                    tracing::error!("[SYNC-WS] Sync triggered by WebSocket failed: {}", e);
                                }
                            }
                        } else {
                            tracing::debug!("[SYNC-WS] Received non-sync message: {}", t);
                        }
                    }
                }
                tracing::warn!("[SYNC-WS] WebSocket connection closed");
            }
            Err(e) => {
                tracing::warn!("[SYNC-WS] Failed to connect to WebSocket: {}, retrying in {}s", e, backoff_secs);
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs)).await;
        backoff_secs = (backoff_secs * 2).min(60);
    }
}
