use axum::{
    body::Bytes,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures::{SinkExt, StreamExt};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};

use crate::models::{
    EncryptedChange, PullCursor, PullResponse, PushRequest, RegisterRequest, RegisterResponse,
};
use crate::storage::{DeviceRow, PushAcceptance, Storage};
use subtle::ConstantTimeEq;

#[derive(Clone, Debug, Serialize)]
pub struct ConnectedDevice {
    pub device_id: String,
    pub hostname: Option<String>,
    pub connected_at: i64,
    pub last_seen: i64,
}

pub type ConnectedDevices = Arc<RwLock<HashMap<String, ConnectedDevice>>>;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
    pub broadcast: broadcast::Sender<String>,
    pub connected_devices: ConnectedDevices,
    pub server_seed_phrase: Arc<str>,
}

#[derive(Debug, Deserialize)]
pub struct PullQuery {
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub hostname: Option<String>,
}

pub fn router(
    storage: Arc<Storage>,
    broadcast_tx: broadcast::Sender<String>,
    server_seed_phrase: Arc<str>,
) -> Router {
    let state = AppState {
        storage,
        broadcast: broadcast_tx,
        connected_devices: Arc::new(RwLock::new(HashMap::new())),
        server_seed_phrase,
    };
    Router::new()
        .route("/health", get(health))
        .route("/register", post(register))
        .route("/push", post(push))
        .route("/pull", get(pull))
        .route("/ws", get(ws_handler))
        .route("/devices", get(get_devices))
        .route(
            "/media/:hash",
            get(get_media).put(put_media).head(head_media),
        )
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

fn verify_passphrase(got: &str, expected: &str) -> bool {
    let a = got.as_bytes();
    let b = expected.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    value.strip_prefix("Bearer ")
}

fn authenticated_device_id(headers: &HeaderMap) -> Option<&str> {
    headers.get("x-aether-device-id")?.to_str().ok()
}

fn require_auth(headers: &HeaderMap, storage: &Storage) -> Result<String, StatusCode> {
    let device_id = authenticated_device_id(headers)
        .filter(|id| !id.is_empty())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let token = bearer_token(headers)
        .filter(|token| !token.is_empty())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    match storage.authenticate_device(device_id, token) {
        Ok(true) => Ok(device_id.to_string()),
        Ok(false) => Err(StatusCode::UNAUTHORIZED),
        Err(e) => {
            tracing::error!("authenticate_device: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn register(
    State(s): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> impl IntoResponse {
    if !verify_passphrase(&body.server_seed_phrase, &s.server_seed_phrase) {
        tracing::warn!(
            "Register rejected: wrong server seed phrase for device {}",
            body.device_id
        );
        return (StatusCode::UNAUTHORIZED, "wrong server seed phrase").into_response();
    }

    let mut token_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut token_bytes);
    let device_token = BASE64.encode(token_bytes);

    if let Err(e) =
        s.storage
            .register_device(&body.device_id, body.hostname.as_deref(), &device_token)
    {
        tracing::error!("register_device: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
    }

    let salt = match s.storage.get_salt() {
        Ok(salt) => salt,
        Err(e) => {
            tracing::error!("get_salt during register: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "salt not configured").into_response();
        }
    };

    tracing::info!("Device {} enrolled", body.device_id);
    (
        StatusCode::OK,
        Json(RegisterResponse { device_token, salt }),
    )
        .into_response()
}

async fn push(
    State(s): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PushRequest>,
) -> impl IntoResponse {
    let device_id = match require_auth(&headers, &s.storage) {
        Ok(id) => id,
        Err(status) => return (status, "unauthorized").into_response(),
    };

    if body.batch_id.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "missing batch_id").into_response();
    }

    let mut decoded_changes = Vec::with_capacity(body.changes.len());
    for ec in &body.changes {
        let nonce = match BASE64.decode(&ec.nonce) {
            Ok(v) => v,
            Err(_) => return (StatusCode::BAD_REQUEST, "invalid nonce base64").into_response(),
        };
        let ct = match BASE64.decode(&ec.ciphertext) {
            Ok(v) => v,
            Err(_) => {
                return (StatusCode::BAD_REQUEST, "invalid ciphertext base64").into_response()
            }
        };
        decoded_changes.push((nonce, ct));
    }

    match s.storage.record_push_if_new(
        &device_id,
        &body.batch_id,
        Some(&body.device_hostname),
        &decoded_changes,
    ) {
        Ok(PushAcceptance::Accepted) => {
            let now = epoch_millis();
            let _ = s.storage.update_device_last_sync(&device_id, now);
            let receiver_count = s.broadcast.send(device_id.clone()).unwrap_or(0);
            tracing::info!(
                "Push from device {}: {} changes, notified {} connected devices",
                device_id,
                body.changes.len(),
                receiver_count
            );
            (StatusCode::OK, "{}").into_response()
        }
        Ok(PushAcceptance::Duplicate) => {
            tracing::info!(
                "Ignoring duplicate batch {} from device {}",
                body.batch_id,
                device_id
            );
            (StatusCode::OK, "{}").into_response()
        }
        Err(e) => {
            tracing::error!("record_push_if_new: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response()
        }
    }
}

async fn ws_handler(
    State(s): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let device_id = match require_auth(&headers, &s.storage) {
        Ok(id) => id,
        Err(status) => return (status, "unauthorized").into_response(),
    };
    let hostname = query.hostname.clone();
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
    let now = epoch_millis();

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

    let device_id_for_recv = my_device_id.clone();
    let connected_devices_for_recv = connected_devices.clone();
    let storage_for_recv = storage.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Pong(_)) => {
                    let now = epoch_millis();
                    let mut devices = connected_devices_for_recv.write().await;
                    if let Some(device) = devices.get_mut(&device_id_for_recv) {
                        device.last_seen = now;
                    }
                    let _ = storage_for_recv.update_device_last_seen(&device_id_for_recv, now);
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    tracing::warn!("WebSocket error from device {}: {}", device_id_for_recv, e);
                    break;
                }
                _ => {}
            }
        }
    });

    let device_id_for_send = my_device_id.clone();
    let mut send_task = tokio::spawn(async move {
        let mut ping_interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            tokio::select! {
                _ = ping_interval.tick() => {
                    if sender.send(Message::Ping(Vec::new().into())).await.is_err() {
                        break;
                    }
                }
                result = rx.recv() => {
                    match result {
                        Ok(pusher_device_id) => {
                            if pusher_device_id != device_id_for_send
                                && sender.send(Message::Text("sync".into())).await.is_err()
                            {
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("Device {} lagged {} messages", device_id_for_send, n);
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = &mut recv_task => send_task.abort(),
        _ = &mut send_task => recv_task.abort(),
    }

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

#[derive(Serialize)]
struct DeviceInfo {
    id: String,
    hostname: Option<String>,
    created_at: i64,
    last_seen: i64,
    last_sync: i64,
    connected: bool,
}

#[derive(Serialize)]
struct DevicesResponse {
    devices: Vec<DeviceInfo>,
    count: usize,
}

async fn get_devices(State(s): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if let Err(status) = require_auth(&headers, &s.storage) {
        return (
            status,
            Json(DevicesResponse {
                devices: vec![],
                count: 0,
            }),
        )
            .into_response();
    }

    let rows = match s.storage.list_devices() {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("list_devices: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DevicesResponse {
                    devices: vec![],
                    count: 0,
                }),
            )
                .into_response();
        }
    };
    let connected = s.connected_devices.read().await;
    let devices: Vec<DeviceInfo> = rows
        .into_iter()
        .map(|r: DeviceRow| DeviceInfo {
            id: r.id.clone(),
            hostname: r.hostname.clone(),
            created_at: r.created_at,
            last_seen: r.last_seen,
            last_sync: r.last_sync,
            connected: connected.contains_key(&r.id),
        })
        .collect();
    let count = devices.len();
    (StatusCode::OK, Json(DevicesResponse { devices, count })).into_response()
}

async fn pull(
    State(s): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<PullQuery>,
) -> impl IntoResponse {
    let device_id = match require_auth(&headers, &s.storage) {
        Ok(id) => id,
        Err(status) => {
            return (
                status,
                Json(PullResponse {
                    changes: vec![],
                    next_cursor: None,
                    has_more: false,
                }),
            )
                .into_response()
        }
    };

    let cursor = match q.cursor.as_deref() {
        Some(raw) => match decode_cursor(raw) {
            Some(cursor) => Some(cursor),
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(PullResponse {
                        changes: vec![],
                        next_cursor: None,
                        has_more: false,
                    }),
                )
                    .into_response()
            }
        },
        None => None,
    };

    let limit = 500i64;
    let rows = match s.storage.pull(cursor.as_ref(), limit) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("pull db: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PullResponse {
                    changes: vec![],
                    next_cursor: cursor,
                    has_more: false,
                }),
            )
                .into_response();
        }
    };

    let changes: Vec<EncryptedChange> = rows
        .iter()
        .map(|row| EncryptedChange {
            nonce: BASE64.encode(&row.nonce),
            ciphertext: BASE64.encode(&row.ciphertext),
        })
        .collect();

    let next_cursor = rows.last().map(|row| PullCursor {
        received_at: row.received_at,
        change_id: row.id,
    });

    let has_more = match next_cursor.as_ref() {
        Some(cursor) => s.storage.has_more_after(cursor).unwrap_or(false),
        None => false,
    };

    if let Some(ref cursor) = next_cursor {
        let _ = s
            .storage
            .update_device_last_sync(&device_id, cursor.received_at);
    }

    (
        StatusCode::OK,
        Json(PullResponse {
            changes,
            next_cursor,
            has_more,
        }),
    )
        .into_response()
}

async fn put_media(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(hash): Path<String>,
    body: Bytes,
) -> impl IntoResponse {
    if let Err(status) = require_auth(&headers, &s.storage) {
        return status;
    }
    if let Err(e) = s.storage.put_blob(&hash, &body) {
        tracing::error!("put_blob: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}

async fn get_media(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    if let Err(status) = require_auth(&headers, &s.storage) {
        return status.into_response();
    }
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
    headers: HeaderMap,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    if let Err(status) = require_auth(&headers, &s.storage) {
        return status;
    }
    if s.storage.has_blob(&hash) {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

fn decode_cursor(raw: &str) -> Option<PullCursor> {
    let (received_at, change_id) = raw.split_once(':')?;
    Some(PullCursor {
        received_at: received_at.parse().ok()?,
        change_id: change_id.parse().ok()?,
    })
}

fn epoch_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}
