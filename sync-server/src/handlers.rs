use axum::{
    body::Bytes,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        DefaultBodyLimit, Path, Query, State,
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
use std::fmt::Display;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};

use crate::models::{
    EncryptedChange, PullCursor, PullResponse, PushRequest, RegisterRequest, RegisterResponse,
};
use crate::storage::{valid_media_hash, DeviceRow, PushAcceptance, Storage};
use subtle::ConstantTimeEq;

const MAX_DEVICE_ID_LEN: usize = 128;
const MAX_HOSTNAME_LEN: usize = 255;
const MAX_BATCH_ID_LEN: usize = 128;
const MAX_CHANGES_PER_PUSH: usize = 1_000;
const MAX_NONCE_BYTES: usize = 24;
const MAX_CIPHERTEXT_BYTES: usize = 1_048_576;
const MAX_API_BODY_BYTES: usize = 8 * 1024 * 1024;
const MAX_MEDIA_BODY_BYTES: usize = 256 * 1024 * 1024;

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
            get(get_media)
                .put(put_media)
                .head(head_media)
                .layer(DefaultBodyLimit::max(MAX_MEDIA_BODY_BYTES)),
        )
        .layer(DefaultBodyLimit::max(MAX_API_BODY_BYTES))
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

async fn storage_blocking<T, E, F>(f: F) -> Result<T, String>
where
    T: Send + 'static,
    E: Display + Send + 'static,
    F: FnOnce() -> Result<T, E> + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| format!("storage task join: {}", e))?
        .map_err(|e| e.to_string())
}

async fn require_auth(headers: &HeaderMap, storage: Arc<Storage>) -> Result<String, StatusCode> {
    let started = Instant::now();
    let device_id = authenticated_device_id(headers)
        .filter(|id| valid_device_id(id))
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_string();
    let token = bearer_token(headers)
        .filter(|token| !token.is_empty())
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_string();
    let device_id_for_auth = device_id.clone();
    match storage_blocking(move || storage.authenticate_device(&device_id_for_auth, &token)).await {
        Ok(true) => {
            tracing::info!(
                "[SYNC-SERVER-TIMING] auth={}ms device_id={}",
                started.elapsed().as_millis(),
                device_id
            );
            Ok(device_id)
        }
        Ok(false) => {
            tracing::info!(
                "[SYNC-SERVER-TIMING] auth={}ms unauthorized_device_id={}",
                started.elapsed().as_millis(),
                device_id
            );
            Err(StatusCode::UNAUTHORIZED)
        }
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
    if !valid_device_id(&body.device_id) {
        return (StatusCode::BAD_REQUEST, "invalid device_id").into_response();
    }
    if body
        .hostname
        .as_deref()
        .is_some_and(|hostname| hostname.len() > MAX_HOSTNAME_LEN)
    {
        return (StatusCode::BAD_REQUEST, "hostname too long").into_response();
    }

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

    let register_started = Instant::now();
    let storage = s.storage.clone();
    let device_id_for_register = body.device_id.clone();
    let hostname_for_register = body.hostname.clone();
    let device_token_for_register = device_token.clone();
    if let Err(e) = storage_blocking(move || {
        storage.register_device(
            &device_id_for_register,
            hostname_for_register.as_deref(),
            &device_token_for_register,
        )
    })
    .await
    {
        tracing::error!("register_device: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
    }
    tracing::info!(
        "[SYNC-SERVER-TIMING] register_device={}ms device_id={}",
        register_started.elapsed().as_millis(),
        body.device_id
    );

    let storage = s.storage.clone();
    let salt = match storage_blocking(move || storage.get_salt()).await {
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
    let device_id = match require_auth(&headers, s.storage.clone()).await {
        Ok(id) => id,
        Err(status) => return (status, "unauthorized").into_response(),
    };

    if body.batch_id.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "missing batch_id").into_response();
    }
    if !valid_batch_id(&body.batch_id) {
        return (StatusCode::BAD_REQUEST, "invalid batch_id").into_response();
    }
    if body.device_hostname.len() > MAX_HOSTNAME_LEN {
        return (StatusCode::BAD_REQUEST, "hostname too long").into_response();
    }
    if body.changes.len() > MAX_CHANGES_PER_PUSH {
        return (StatusCode::PAYLOAD_TOO_LARGE, "too many changes").into_response();
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
        if nonce.is_empty() || nonce.len() > MAX_NONCE_BYTES {
            return (StatusCode::BAD_REQUEST, "invalid nonce length").into_response();
        }
        if ct.is_empty() || ct.len() > MAX_CIPHERTEXT_BYTES {
            return (StatusCode::BAD_REQUEST, "invalid ciphertext length").into_response();
        }
        decoded_changes.push((nonce, ct));
    }

    let change_count = decoded_changes.len();
    let record_started = Instant::now();
    let storage = s.storage.clone();
    let device_id_for_record = device_id.clone();
    let batch_id_for_record = body.batch_id.clone();
    let hostname_for_record = body.device_hostname.clone();
    match storage_blocking(move || {
        storage.record_push_if_new(
            &device_id_for_record,
            &batch_id_for_record,
            Some(&hostname_for_record),
            &decoded_changes,
        )
    })
    .await
    {
        Ok(PushAcceptance::Accepted) => {
            tracing::info!(
                "[SYNC-SERVER-TIMING] record_push={}ms changes={} accepted=true",
                record_started.elapsed().as_millis(),
                change_count
            );
            let now = epoch_millis();
            let storage = s.storage.clone();
            let device_id_for_update = device_id.clone();
            let _ = storage_blocking(move || {
                storage.update_device_last_sync(&device_id_for_update, now)
            })
            .await;
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
                "[SYNC-SERVER-TIMING] record_push={}ms changes={} accepted=false duplicate=true",
                record_started.elapsed().as_millis(),
                change_count
            );
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
    let device_id = match require_auth(&headers, s.storage.clone()).await {
        Ok(id) => id,
        Err(status) => return (status, "unauthorized").into_response(),
    };
    let hostname = query.hostname.clone();
    if hostname
        .as_deref()
        .is_some_and(|hostname| hostname.len() > MAX_HOSTNAME_LEN)
    {
        return (StatusCode::BAD_REQUEST, "hostname too long").into_response();
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
    let storage_for_seen = storage.clone();
    let device_id_for_seen = my_device_id.clone();
    let _ = storage_blocking(move || {
        storage_for_seen.update_device_last_seen(&device_id_for_seen, now)
    })
    .await;

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
                    let storage_for_seen = storage_for_recv.clone();
                    let device_id_for_seen = device_id_for_recv.clone();
                    let _ = storage_blocking(move || {
                        storage_for_seen.update_device_last_seen(&device_id_for_seen, now)
                    })
                    .await;
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
    if let Err(status) = require_auth(&headers, s.storage.clone()).await {
        return (
            status,
            Json(DevicesResponse {
                devices: vec![],
                count: 0,
            }),
        )
            .into_response();
    }

    let storage = s.storage.clone();
    let rows = match storage_blocking(move || storage.list_devices()).await {
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
    let device_id = match require_auth(&headers, s.storage.clone()).await {
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

    let limit = 500usize;
    let pull_started = Instant::now();
    let storage = s.storage.clone();
    let cursor_for_pull = cursor.clone();
    let mut rows =
        match storage_blocking(move || storage.pull(cursor_for_pull.as_ref(), (limit + 1) as i64))
            .await
        {
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
    let has_more = rows.len() > limit;
    if has_more {
        rows.truncate(limit);
    }
    tracing::info!(
        "[SYNC-SERVER-TIMING] pull_query={}ms rows={}",
        pull_started.elapsed().as_millis(),
        rows.len()
    );

    let encode_started = Instant::now();
    let changes: Vec<EncryptedChange> = rows
        .iter()
        .map(|row| EncryptedChange {
            nonce: BASE64.encode(&row.nonce),
            ciphertext: BASE64.encode(&row.ciphertext),
        })
        .collect();
    tracing::info!(
        "[SYNC-SERVER-TIMING] pull_encode={}ms changes={}",
        encode_started.elapsed().as_millis(),
        changes.len()
    );

    let next_cursor = rows.last().map(|row| PullCursor {
        received_at: row.received_at,
        change_id: row.id,
    });

    let has_more_started = Instant::now();
    tracing::info!(
        "[SYNC-SERVER-TIMING] has_more={}ms has_more={}",
        has_more_started.elapsed().as_millis(),
        has_more
    );

    if let Some(ref cursor) = next_cursor {
        let update_started = Instant::now();
        let storage = s.storage.clone();
        let device_id_for_update = device_id.clone();
        let received_at = cursor.received_at;
        let _ = storage_blocking(move || {
            storage.update_device_last_sync(&device_id_for_update, received_at)
        })
        .await;
        tracing::info!(
            "[SYNC-SERVER-TIMING] last_sync_update={}ms device_id={}",
            update_started.elapsed().as_millis(),
            device_id
        );
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
    if let Err(status) = require_auth(&headers, s.storage.clone()).await {
        return status;
    }
    if !valid_media_hash(&hash) {
        return StatusCode::BAD_REQUEST;
    }
    let started = Instant::now();
    let storage = s.storage.clone();
    let hash_for_put = hash.clone();
    let body_bytes = body.to_vec();
    let body_len = body_bytes.len();
    if let Err(e) = storage_blocking(move || storage.put_blob(&hash_for_put, &body_bytes)).await {
        tracing::error!("put_blob: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    tracing::info!(
        "[SYNC-SERVER-TIMING] blob_put={}ms hash={} bytes={}",
        started.elapsed().as_millis(),
        hash,
        body_len
    );
    StatusCode::OK
}

async fn get_media(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    if let Err(status) = require_auth(&headers, s.storage.clone()).await {
        return status.into_response();
    }
    if !valid_media_hash(&hash) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let started = Instant::now();
    let storage = s.storage.clone();
    let hash_for_get = hash.clone();
    match storage_blocking(move || storage.get_blob(&hash_for_get)).await {
        Ok(Some(data)) => {
            tracing::info!(
                "[SYNC-SERVER-TIMING] blob_get={}ms hash={} bytes={}",
                started.elapsed().as_millis(),
                hash,
                data.len()
            );
            (StatusCode::OK, bytes::Bytes::from(data)).into_response()
        }
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
    if let Err(status) = require_auth(&headers, s.storage.clone()).await {
        return status;
    }
    if !valid_media_hash(&hash) {
        return StatusCode::BAD_REQUEST;
    }
    let storage = s.storage.clone();
    let exists =
        storage_blocking(move || Ok::<_, std::convert::Infallible>(storage.has_blob(&hash)))
            .await
            .unwrap_or(false);
    if exists {
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

fn valid_device_id(device_id: &str) -> bool {
    !device_id.trim().is_empty()
        && device_id.len() <= MAX_DEVICE_ID_LEN
        && device_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_batch_id(batch_id: &str) -> bool {
    !batch_id.trim().is_empty()
        && batch_id.len() <= MAX_BATCH_ID_LEN
        && batch_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn epoch_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::{decode_cursor, valid_batch_id, valid_device_id};

    #[test]
    fn decodes_cursor_pair() {
        let cursor = decode_cursor("1700000000000:42").expect("cursor should decode");

        assert_eq!(cursor.received_at, 1_700_000_000_000);
        assert_eq!(cursor.change_id, 42);
    }

    #[test]
    fn rejects_invalid_cursors() {
        assert!(decode_cursor("").is_none());
        assert!(decode_cursor("1700000000000").is_none());
        assert!(decode_cursor("abc:42").is_none());
        assert!(decode_cursor("1700000000000:abc").is_none());
    }

    #[test]
    fn validates_device_ids() {
        assert!(valid_device_id("device-123_abc"));
        assert!(!valid_device_id(""));
        assert!(!valid_device_id("   "));
        assert!(!valid_device_id("../device"));
        assert!(!valid_device_id("device id"));
        assert!(!valid_device_id(&"a".repeat(129)));
    }

    #[test]
    fn validates_batch_ids() {
        assert!(valid_batch_id("batch-123_abc"));
        assert!(!valid_batch_id(""));
        assert!(!valid_batch_id("../batch"));
        assert!(!valid_batch_id("batch id"));
        assert!(!valid_batch_id(&"a".repeat(129)));
    }
}
