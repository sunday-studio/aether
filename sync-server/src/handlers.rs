use axum::{
    body::Bytes,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, head, post, put},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

use crate::models::{EncryptedChange, PullResponse, PushRequest};
use crate::storage::Storage;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
    /// Broadcasts the device_id that pushed, so other devices can sync
    pub broadcast: broadcast::Sender<String>,
}

#[derive(Debug, Deserialize)]
pub struct PullQuery {
    pub since: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub device_id: Option<String>,
}

pub fn router(storage: Arc<Storage>, broadcast_tx: broadcast::Sender<String>) -> Router {
    let state = AppState {
        storage,
        broadcast: broadcast_tx,
    };
    Router::new()
        .route("/health", get(health))
        .route("/salt", get(get_salt))
        .route("/push", post(push))
        .route("/pull", get(pull))
        .route("/ws", get(ws_handler))
        .route("/media/:hash", get(get_media).put(put_media).head(head_media))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

#[derive(Serialize)]
struct SaltResponse {
    salt: String,
}

async fn get_salt(State(s): State<AppState>) -> impl IntoResponse {
    match s.storage.get_salt() {
        Ok(salt) => (StatusCode::OK, Json(SaltResponse { salt })).into_response(),
        Err(e) => {
            tracing::error!("get_salt: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "salt not configured").into_response()
        }
    }
}

async fn push(State(s): State<AppState>, Json(body): Json<PushRequest>) -> impl IntoResponse {
    for ec in &body.changes {
        let nonce = match BASE64.decode(&ec.nonce) {
            Ok(v) => v,
            Err(_) => return (StatusCode::BAD_REQUEST, "invalid nonce base64").into_response(),
        };
        let ct = match BASE64.decode(&ec.ciphertext) {
            Ok(v) => v,
            Err(_) => return (StatusCode::BAD_REQUEST, "invalid ciphertext base64").into_response(),
        };
        if let Err(e) = s.storage.push(&body.device_id, Some(&body.device_hostname), &nonce, &ct) {
            tracing::error!("push db: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
        }
    }
    // Broadcast the device_id that pushed, so other devices know to sync
    let receiver_count = s.broadcast.send(body.device_id.clone()).unwrap_or(0);
    tracing::info!(
        "Push from device {}: {} changes, notified {} connected devices",
        body.device_id,
        body.changes.len(),
        receiver_count
    );
    (StatusCode::OK, "{}").into_response()
}

async fn ws_handler(
    State(s): State<AppState>,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let device_id = query.device_id.unwrap_or_else(|| "unknown".to_string());
    let rx = s.broadcast.subscribe();
    tracing::info!("WebSocket connection from device: {}", device_id);
    ws.on_upgrade(move |socket| handle_websocket(socket, rx, device_id))
}

async fn handle_websocket(
    socket: WebSocket,
    mut rx: broadcast::Receiver<String>,
    my_device_id: String,
) {
    let (mut sender, mut receiver) = socket.split();

    // Task to read from client (handle pongs, detect disconnect)
    let device_id_for_recv = my_device_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Pong(_)) => {
                    tracing::debug!("Received pong from device {}", device_id_for_recv);
                }
                Ok(Message::Close(_)) => {
                    tracing::info!("WebSocket closed by device {}", device_id_for_recv);
                    break;
                }
                Err(e) => {
                    tracing::warn!("WebSocket error from device {}: {}", device_id_for_recv, e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Task to send pings and sync notifications
    let device_id_for_send = my_device_id.clone();
    let mut send_task = tokio::spawn(async move {
        let mut ping_interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            tokio::select! {
                _ = ping_interval.tick() => {
                    if sender.send(Message::Ping(vec![])).await.is_err() {
                        tracing::debug!("Failed to send ping to device {}", device_id_for_send);
                        break;
                    }
                }
                result = rx.recv() => {
                    match result {
                        Ok(pusher_device_id) => {
                            // Only notify if a DIFFERENT device pushed
                            if pusher_device_id != device_id_for_send {
                                tracing::info!(
                                    "Notifying device {} about changes from device {}",
                                    device_id_for_send,
                                    pusher_device_id
                                );
                                if sender.send(Message::Text("sync".into())).await.is_err() {
                                    tracing::debug!("Failed to send sync to device {}", device_id_for_send);
                                    break;
                                }
                            } else {
                                tracing::debug!(
                                    "Skipping notification to device {} (same as pusher)",
                                    device_id_for_send
                                );
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("Device {} lagged {} messages", device_id_for_send, n);
                            // Continue, we'll catch up on next broadcast
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            tracing::info!("Broadcast channel closed");
                            break;
                        }
                    }
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut recv_task => {
            send_task.abort();
        }
        _ = &mut send_task => {
            recv_task.abort();
        }
    }

    tracing::info!("WebSocket connection closed for device {}", my_device_id);
}

async fn pull(
    State(s): State<AppState>,
    Query(q): Query<PullQuery>,
) -> impl IntoResponse {
    let since = q.since.unwrap_or(0);
    let limit = 500i64;
    let rows = match s.storage.pull(since, limit) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("pull db: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PullResponse { changes: vec![], timestamp: since, has_more: false }),
            )
                .into_response();
        }
    };
    let changes: Vec<EncryptedChange> = rows
        .into_iter()
        .map(|(nonce, ct)| EncryptedChange {
            nonce: BASE64.encode(&nonce),
            ciphertext: BASE64.encode(&ct),
        })
        .collect();
    let ts = s.storage.max_received_at().unwrap_or(since);
    let has_more = (changes.len() as i64) >= limit;
    (StatusCode::OK, Json(PullResponse { changes, timestamp: ts, has_more })).into_response()
}

async fn put_media(
    State(s): State<AppState>,
    axum::extract::Path(hash): axum::extract::Path<String>,
    body: Bytes,
) -> impl IntoResponse {
    if let Err(e) = s.storage.put_blob(&hash, &body) {
        tracing::error!("put_blob: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}

async fn get_media(
    State(s): State<AppState>,
    axum::extract::Path(hash): axum::extract::Path<String>,
) -> impl IntoResponse {
    match s.storage.get_blob(&hash) {
        Ok(Some(data)) => (StatusCode::OK, bytes::Bytes::from(data)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("get_blob: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn head_media(
    State(s): State<AppState>,
    axum::extract::Path(hash): axum::extract::Path<String>,
) -> impl IntoResponse {
    if s.storage.has_blob(&hash) {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}
