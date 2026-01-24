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
    let mut backoff_secs = 1u64;
    loop {
        let url = engine.try_get_url();
        let Some(ws_url) = url else {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            backoff_secs = 1;
            continue;
        };
        let ws_url = to_ws_url(&ws_url);
        match connect_async(&ws_url).await {
            Ok((mut ws, _)) => {
                backoff_secs = 1;
                while let Some(msg) = ws.next().await {
                    let Ok(msg) = msg else { break };
                    if let Message::Text(t) = msg {
                        if t.trim() == "sync" {
                            if let Ok(status) = engine.sync().await {
                                let _ = app.emit("sync-status", &status);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::debug!("ws disconnect: {}", e);
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs)).await;
        backoff_secs = (backoff_secs * 2).min(60);
    }
}
