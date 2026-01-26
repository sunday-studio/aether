use crate::commands::params::{EmptyPathParams, EmptyQueryParams, EmptyRequest, MediaIdPathParams};
use crate::db::connection;
use crate::error::{AppError, Result};
use crate::settings;
use crate::sync::{self, SyncEngine, SyncStatus};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConfigureSyncRequest {
    pub server_url: String,
    pub passphrase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReconnectSyncRequest {
    pub passphrase: String,
}

/// Configure sync with server URL and passphrase
#[utoipa::path(
    post,
    path = "/v1/sync/configure",
    tag = "Sync",
    request_body = ConfigureSyncRequest,
    responses(
        (status = 200, description = "Sync configured successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn configure_sync(
    app: AppHandle,
    engine: State<'_, Arc<SyncEngine>>,
    request_data: Option<ConfigureSyncRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<()> {
    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    if request.server_url.is_empty() {
        return Err(AppError::BadRequest("Server URL is required".to_string()));
    }
    if request.passphrase.len() < 12 {
        return Err(AppError::BadRequest("Passphrase must be at least 12 characters".to_string()));
    }
    
    engine
        .configure(request.server_url, request.passphrase)
        .await
        .map_err(|e| AppError::Sync(e.to_string()))?;
    if let Ok(status) = engine.status().await {
        let _ = app.emit("sync-status", &status);
    }
    Ok(())
}

/// Trigger a sync operation (pull, apply, push)
#[utoipa::path(
    post,
    path = "/v1/sync/now",
    tag = "Sync",
    responses(
        (status = 200, description = "Sync completed", body = SyncStatus),
        (status = 400, description = "Bad request - sync not configured"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn sync_now(
    app: AppHandle,
    engine: State<'_, Arc<SyncEngine>>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<SyncStatus> {
    let status = engine.sync().await.map_err(|e| AppError::Sync(e.to_string()))?;
    let _ = app.emit("sync-status", &status);
    Ok(status)
}

/// Get current sync status
#[utoipa::path(
    get,
    path = "/v1/sync/status",
    tag = "Sync",
    responses(
        (status = 200, description = "Current sync status", body = SyncStatus),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_sync_status(
    engine: State<'_, Arc<SyncEngine>>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<SyncStatus> {
    engine.status().await.map_err(|e| AppError::Sync(e.to_string()))
}

/// Disconnect sync (clear configuration)
#[utoipa::path(
    post,
    path = "/v1/sync/disconnect",
    tag = "Sync",
    responses(
        (status = 200, description = "Sync disconnected successfully"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn disconnect_sync(
    engine: State<'_, Arc<SyncEngine>>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<()> {
    engine.disconnect().await.map_err(|e| AppError::Sync(e.to_string()))
}

/// Reconnect sync with passphrase
#[utoipa::path(
    post,
    path = "/v1/sync/reconnect",
    tag = "Sync",
    request_body = ReconnectSyncRequest,
    responses(
        (status = 200, description = "Sync reconnected", body = SyncStatus),
        (status = 400, description = "Bad request - invalid passphrase"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn reconnect_sync(
    app: AppHandle,
    engine: State<'_, Arc<SyncEngine>>,
    request_data: Option<ReconnectSyncRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<SyncStatus> {
    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    if request.passphrase.len() < 12 {
        return Err(AppError::BadRequest("Passphrase must be at least 12 characters".to_string()));
    }
    
    engine.reconnect(request.passphrase).await.map_err(|e| AppError::Sync(e.to_string()))?;
    let status = engine.status().await.map_err(|e| AppError::Sync(e.to_string()))?;
    let _ = app.emit("sync-status", &status);
    Ok(status)
}

/// For on-demand media: ensure the blob is on disk before the frontend reads/plays it.
/// No-op when sync.media_sync_policy is "auto" or sync is not configured.
#[utoipa::path(
    post,
    path = "/v1/sync/media/:mediaId/ensure",
    tag = "Sync",
    params(
        ("mediaId" = String, Path, description = "Media ID")
    ),
    responses(
        (status = 200, description = "Media blob ensured"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn ensure_media_blob(
    db: State<'_, crate::DbState>,
    engine: State<'_, Arc<SyncEngine>>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<MediaIdPathParams>,
) -> Result<()> {
    let media_id = path_params
        .and_then(|p| Some(p.media_id))
        .ok_or_else(|| AppError::BadRequest("Media ID is required".to_string()))?;
    let database = connection::get_database(&*db);
    let url = engine.try_get_url();
    let key = engine.try_get_key().await;
    let policy = settings::get_setting(database.clone(), "sync.media_sync_policy")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "on_demand".to_string());
    sync::media::ensure_media_blob(
        database.as_ref(),
        &media_id,
        url.as_deref(),
        key.as_ref(),
        &policy,
    )
    .await
    .map_err(|e| AppError::Sync(e.to_string()))
}
