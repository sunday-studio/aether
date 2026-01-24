use axum::{
    body::Bytes,
    extract::{
        ws::{Message, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, head, post, put},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::models::{EncryptedChange, PullResponse, PushRequest};
use crate::storage::Storage;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
    pub broadcast: broadcast::Sender<()>,
}

#[derive(Debug, Deserialize)]
pub struct PullQuery {
    pub since: Option<i64>,
}

pub fn router(storage: Arc<Storage>, broadcast_tx: broadcast::Sender<()>) -> Router {
    let state = AppState {
        storage,
        broadcast: broadcast_tx,
    };
    Router::new()
        .route("/health", get(health))
        .route("/push", post(push))
        .route("/pull", get(pull))
        .route("/ws", get(ws_handler))
        .route("/media/:hash", get(get_media).put(put_media).head(head_media))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
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
        if let Err(e) = s.storage.push(&body.device_id, &nonce, &ct) {
            tracing::error!("push db: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
        }
    }
    let _ = s.broadcast.send(());
    (StatusCode::OK, "{}").into_response()
}

async fn ws_handler(State(s): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    let mut rx = s.broadcast.subscribe();
    ws.on_upgrade(move |mut socket| async move {
        while let Ok(()) = rx.recv().await {
            if socket.send(Message::Text("sync".into())).await.is_err() {
                break;
            }
        }
    })
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
