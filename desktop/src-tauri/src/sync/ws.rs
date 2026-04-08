//! WebSocket listener: connect to sync server /ws when configured, run sync on "sync" message.

use crate::sync::SyncEngine;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
};

fn to_ws_url(base: &str, hostname: &str) -> String {
    let s = base.trim_end_matches('/');
    let (scheme, rest) = if s.starts_with("https://") {
        ("wss://", &s[8..])
    } else if s.starts_with("http://") {
        ("ws://", &s[7..])
    } else {
        ("wss://", s)
    };
    let encoded_hostname = urlencoding::encode(hostname);
    format!("{}{}/ws?hostname={}", scheme, rest, encoded_hostname)
}

pub async fn run_ws_listener(engine: Arc<SyncEngine>, app: AppHandle) {
    let mut backoff_secs = 1u64;

    loop {
        let Some(base_url) = engine.try_get_url() else {
            engine.wait_for_url_configured().await;
            backoff_secs = 1;
            continue;
        };

        let device_id = match engine.get_device_id().await {
            Ok(id) => id,
            Err(e) => {
                tracing::warn!("[SYNC-WS] Failed to get device_id: {}, retrying in 10s", e);
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        };
        let device_token = match engine.get_device_token().await {
            Ok(Some(token)) => token,
            Ok(None) => {
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
            Err(e) => {
                tracing::warn!(
                    "[SYNC-WS] Failed to get device token: {}, retrying in 10s",
                    e
                );
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        };
        let hostname = engine
            .get_device_hostname()
            .await
            .unwrap_or_else(|_| "unknown".to_string());

        let ws_url = to_ws_url(&base_url, &hostname);
        let mut request = match ws_url.into_client_request() {
            Ok(request) => request,
            Err(e) => {
                tracing::warn!("[SYNC-WS] Invalid WebSocket request: {}", e);
                tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                backoff_secs = (backoff_secs * 2).min(60);
                continue;
            }
        };
        request
            .headers_mut()
            .insert("x-aether-device-id", device_id.parse().unwrap());
        request.headers_mut().insert(
            "authorization",
            format!("Bearer {}", device_token).parse().unwrap(),
        );

        match connect_async(request).await {
            Ok((ws, _)) => {
                backoff_secs = 1;
                let (mut write, mut read) = ws.split();
                let engine_clone = engine.clone();
                let app_clone = app.clone();

                let ping_handle = tokio::spawn(async move {
                    let mut interval = tokio::time::interval(Duration::from_secs(30));
                    loop {
                        interval.tick().await;
                        if write.send(Message::Ping(Vec::new().into())).await.is_err() {
                            break;
                        }
                    }
                });

                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(t)) if t.trim() == "sync" => {
                            match engine_clone.sync().await {
                                Ok(status) => {
                                    let _ = app_clone.emit("sync-status", &status);
                                }
                                Err(e) => tracing::error!(
                                    "[SYNC-WS] Sync triggered by WebSocket failed: {}",
                                    e
                                ),
                            }
                        }
                        Ok(Message::Close(_)) => break,
                        Err(e) => {
                            tracing::warn!("[SYNC-WS] WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }

                ping_handle.abort();
            }
            Err(e) => {
                tracing::warn!(
                    "[SYNC-WS] Failed to connect to WebSocket: {}, retrying in {}s",
                    e,
                    backoff_secs
                );
            }
        }

        tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
        backoff_secs = (backoff_secs * 2).min(60);
    }
}
