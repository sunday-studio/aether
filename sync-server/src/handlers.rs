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
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};

use crate::models::{EncryptedChange, PullResponse, PushRequest, RegisterRequest};
use crate::storage::Storage;
use subtle::ConstantTimeEq;

/// Tracks a connected device
#[derive(Clone, Debug, Serialize)]
pub struct ConnectedDevice {
    pub device_id: String,
    pub hostname: Option<String>,
    pub connected_at: i64,
    pub last_seen: i64,
}

/// Shared state for tracking connected devices
pub type ConnectedDevices = Arc<RwLock<HashMap<String, ConnectedDevice>>>;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
    /// Broadcasts the device_id that pushed, so other devices can sync
    pub broadcast: broadcast::Sender<String>,
    /// Tracks currently connected WebSocket clients
    pub connected_devices: ConnectedDevices,
    /// When set, only registered devices (matching passphrase) can pull/push/ws
    pub server_passphrase: Option<Arc<str>>,
}

#[derive(Debug, Deserialize)]
pub struct PullQuery {
    pub since: Option<i64>,
    pub device_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub device_id: Option<String>,
    pub hostname: Option<String>,
}

pub fn router(
    storage: Arc<Storage>,
    broadcast_tx: broadcast::Sender<String>,
    server_passphrase: Option<Arc<str>>,
) -> Router {
    let state = AppState {
        storage,
        broadcast: broadcast_tx,
        connected_devices: Arc::new(RwLock::new(HashMap::new())),
        server_passphrase,
    };
    Router::new()
        .route("/health", get(health))
        .route("/salt", get(get_salt))
        .route("/register", post(register))
        .route("/push", post(push))
        .route("/pull", get(pull))
        .route("/ws", get(ws_handler))
        .route("/devices", get(get_devices))
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

fn verify_passphrase(got: &str, expected: &str) -> bool {
    let a = got.as_bytes();
    let b = expected.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

async fn register(State(s): State<AppState>, Json(body): Json<RegisterRequest>) -> impl IntoResponse {
    let Some(ref expected) = s.server_passphrase else {
        if let Err(e) = s.storage.register_device(&body.device_id, body.hostname.as_deref()) {
            tracing::error!("register_device: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
        }
        tracing::info!("Device {} registered (no server passphrase)", body.device_id);
        return (StatusCode::OK, "{}").into_response();
    };
    if !verify_passphrase(&body.passphrase, expected) {
        tracing::warn!("Register rejected: wrong passphrase for device {}", body.device_id);
        return (StatusCode::UNAUTHORIZED, "wrong passphrase").into_response();
    }
    if let Err(e) = s.storage.register_device(&body.device_id, body.hostname.as_deref()) {
        tracing::error!("register_device: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
    }
    tracing::info!("Device {} registered", body.device_id);
    (StatusCode::OK, "{}").into_response()
}

async fn push(State(s): State<AppState>, Json(body): Json<PushRequest>) -> impl IntoResponse {
    if s.server_passphrase.is_some() {
        match s.storage.is_device_registered(&body.device_id) {
            Ok(false) => {
                tracing::warn!("Push rejected: device {} not registered", body.device_id);
                return (StatusCode::UNAUTHORIZED, "device not registered").into_response();
            }
            Err(e) => {
                tracing::error!("is_device_registered: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
            }
            _ => {}
        }
    }
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
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    let _ = s.storage.update_device_last_sync(&body.device_id, now);
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
    let hostname = query.hostname.clone();
    if s.server_passphrase.is_some() {
        match s.storage.is_device_registered(&device_id) {
            Ok(false) => {
                tracing::warn!("WebSocket rejected: device {} not registered", device_id);
                return (StatusCode::UNAUTHORIZED, "device not registered").into_response();
            }
            Err(e) => {
                tracing::error!("is_device_registered: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
            }
            _ => {}
        }
    }
    let rx = s.broadcast.subscribe();
    let connected_devices = s.connected_devices.clone();
    let storage = s.storage.clone();
    tracing::info!(
        "WebSocket connection from device: {} (hostname: {:?})",
        device_id,
        hostname
    );
    ws.on_upgrade(move |socket| {
        handle_websocket(socket, rx, device_id, hostname, connected_devices, storage)
    })
}

async fn handle_websocket(
    socket: WebSocket,
    mut rx: broadcast::Receiver<String>,
    my_device_id: String,
    hostname: Option<String>,
    connected_devices: ConnectedDevices,
    storage: Arc<Storage>,
) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    // Register device as connected
    {
        let mut devices = connected_devices.write().await;
        devices.insert(
            my_device_id.clone(),
            ConnectedDevice {
                device_id: my_device_id.clone(),
                hostname: hostname.clone(),
                connected_at: now,
                last_seen: now,
            },
        );
        tracing::info!(
            "Device {} registered, {} devices now connected",
            my_device_id,
            devices.len()
        );
    }
    let _ = storage.update_device_last_seen(&my_device_id, now);

    let (mut sender, mut receiver) = socket.split();

    // Task to read from client (handle pongs, detect disconnect)
    let device_id_for_recv = my_device_id.clone();
    let connected_devices_for_recv = connected_devices.clone();
    let storage_for_recv = storage.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Pong(_)) => {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as i64;
                    let mut devices = connected_devices_for_recv.write().await;
                    if let Some(device) = devices.get_mut(&device_id_for_recv) {
                        device.last_seen = now;
                    }
                    let _ = storage_for_recv.update_device_last_seen(&device_id_for_recv, now);
                    tracing::debug!("Received pong from device {}, updated last_seen", device_id_for_recv);
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

    // Unregister device on disconnect
    {
        let mut devices = connected_devices.write().await;
        devices.remove(&my_device_id);
        tracing::info!(
            "Device {} disconnected, {} devices now connected",
            my_device_id,
            devices.len()
        );
    }
}

/// Response for /devices endpoint
#[derive(Serialize)]
struct DevicesResponse {
    devices: Vec<ConnectedDevice>,
    count: usize,
}

/// Get list of currently connected devices
async fn get_devices(State(s): State<AppState>) -> impl IntoResponse {
    let devices = s.connected_devices.read().await;
    let device_list: Vec<ConnectedDevice> = devices.values().cloned().collect();
    let count = device_list.len();
    (StatusCode::OK, Json(DevicesResponse { devices: device_list, count }))
}

async fn pull(
    State(s): State<AppState>,
    Query(q): Query<PullQuery>,
) -> impl IntoResponse {
    if s.server_passphrase.is_some() {
        let device_id = match &q.device_id {
            Some(id) if !id.is_empty() => id.as_str(),
            _ => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(PullResponse { changes: vec![], timestamp: q.since.unwrap_or(0), has_more: false }),
                )
                    .into_response();
            }
        };
        match s.storage.is_device_registered(device_id) {
            Ok(false) => {
                tracing::warn!("Pull rejected: device {} not registered", device_id);
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(PullResponse { changes: vec![], timestamp: q.since.unwrap_or(0), has_more: false }),
                )
                    .into_response();
            }
            Err(e) => {
                tracing::error!("is_device_registered: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(PullResponse { changes: vec![], timestamp: q.since.unwrap_or(0), has_more: false }),
                )
                    .into_response();
            }
            _ => {}
        }
    }
    let since = q.since.unwrap_or(0);
    let limit = 500i64;
    let device_id = q.device_id.clone();
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
    if let Some(ref did) = device_id {
        let _ = s.storage.update_device_last_sync(did, ts);
    }
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
