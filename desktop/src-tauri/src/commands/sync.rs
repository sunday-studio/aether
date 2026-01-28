use crate::commands::params::{EmptyPathParams, EmptyQueryParams, EmptyRequest, MediaIdPathParams};
use crate::db::connection;
use crate::error::{AppError, Result};
use crate::settings;
use crate::sync::{self, SyncEngine, SyncStatus};
use crate::sync::metadata;
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
    
    tracing::info!("[SYNC-CMD] Configuring sync via command");
    engine
        .configure(&app, request.server_url, request.passphrase)
        .await
        .map_err(|e| {
            tracing::error!("[SYNC-CMD] Configuration failed: {}", e);
            AppError::Sync(e.to_string())
        })?;
    if let Ok(status) = engine.status().await {
        tracing::info!("[SYNC-CMD] Configuration complete, emitting status event");
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
    tracing::info!("[SYNC-CMD] sync_now command invoked");
    let status = engine.sync().await.map_err(|e| {
        tracing::error!("[SYNC-CMD] sync_now failed: {}", e);
        AppError::Sync(e.to_string())
    })?;
    tracing::info!("[SYNC-CMD] sync_now completed, emitting status event");
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
    app: AppHandle,
    engine: State<'_, Arc<SyncEngine>>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<()> {
    engine.disconnect(&app).await.map_err(|e| AppError::Sync(e.to_string()))
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

/// Diagnostic command to check sync trigger state
#[tauri::command]
pub async fn check_sync_triggers(
    db: State<'_, crate::DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<serde_json::Value> {
    let database = connection::get_database(&*db);
    let conn = database.connect().map_err(AppError::LibSQL)?;
    
    // Check _suppress_triggers
    let suppress_triggers = metadata::get_suppress_triggers(database.as_ref()).await?;
    tracing::info!("[SYNC-DIAG] _suppress_triggers = '{}'", suppress_triggers);
    
    // Check outbox count
    let mut rows = conn
        .query("SELECT COUNT(*) FROM _sync_outbox", libsql::params![])
        .await
        .map_err(AppError::LibSQL)?;
    let outbox_count: i64 = if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
        row.get(0).unwrap_or(0)
    } else {
        0
    };
    tracing::info!("[SYNC-DIAG] Outbox count: {}", outbox_count);
    
    // Check if triggers exist
    let mut trigger_rows = conn
        .query("SELECT name FROM sqlite_master WHERE type='trigger' AND name LIKE '%_sync_%'", libsql::params![])
        .await
        .map_err(AppError::LibSQL)?;
    let mut triggers = Vec::new();
    while let Some(row) = trigger_rows.next().await.map_err(AppError::LibSQL)? {
        if let Ok(name) = row.get::<String>(0) {
            triggers.push(name);
        }
    }
    tracing::info!("[SYNC-DIAG] Found {} sync triggers", triggers.len());
    
    // Check a sample entry to see if _sync_id and _updated_at are set
    let mut entry_rows = conn
        .query("SELECT id, _sync_id, _updated_at FROM entries LIMIT 1", libsql::params![])
        .await
        .map_err(AppError::LibSQL)?;
    let mut sample_entry: Option<serde_json::Value> = None;
    if let Some(row) = entry_rows.next().await.map_err(AppError::LibSQL)? {
        let id: Option<String> = row.get(0).ok();
        let sync_id: Option<String> = row.get(1).ok();
        let updated_at: Option<i64> = row.get(2).ok();
        sample_entry = Some(serde_json::json!({
            "id": id,
            "_sync_id": sync_id,
            "_updated_at": updated_at,
        }));
    }
    
    Ok(serde_json::json!({
        "suppress_triggers": suppress_triggers,
        "outbox_count": outbox_count,
        "trigger_count": triggers.len(),
        "triggers": triggers,
        "sample_entry": sample_entry,
    }))
}

/// Test command to manually trigger an outbox entry (for debugging)
#[tauri::command]
pub async fn test_sync_trigger(
    db: State<'_, crate::DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<serde_json::Value> {
    let database = connection::get_database(&*db);
    let conn = database.connect().map_err(AppError::LibSQL)?;
    
    // Get first entry
    let mut rows = conn
        .query("SELECT id, _sync_id, _updated_at FROM entries LIMIT 1", libsql::params![])
        .await
        .map_err(AppError::LibSQL)?;
    
    let entry_id = if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
        row.get::<String>(0).map_err(AppError::LibSQL)?
    } else {
        return Err(AppError::BadRequest("No entries found to test".to_string()));
    };
    
    let entry_id_clone = entry_id.clone();
    
    // Get entry details before update
    let mut entry_details = conn
        .query("SELECT id, _sync_id, _updated_at FROM entries WHERE id = ?1", libsql::params![entry_id_clone.as_str()])
        .await
        .map_err(AppError::LibSQL)?;
    let (sync_id, old_updated_at) = if let Some(row) = entry_details.next().await.map_err(AppError::LibSQL)? {
        let sync_id: Option<String> = row.get(1).ok();
        let updated_at: Option<i64> = row.get(2).ok();
        (sync_id, updated_at)
    } else {
        return Err(AppError::BadRequest("Entry not found".to_string()));
    };
    
    tracing::info!("[SYNC-TEST] Testing trigger by updating entry: {} (sync_id: {:?}, old_updated_at: {:?})", entry_id_clone, sync_id, old_updated_at);
    
    // Check if triggers exist and get their SQL
    let mut trigger_rows = conn
        .query("SELECT name, sql FROM sqlite_master WHERE type='trigger' AND name LIKE '%entries_sync%'", libsql::params![])
        .await
        .map_err(AppError::LibSQL)?;
    let mut triggers = Vec::new();
    let mut trigger_sqls = Vec::new();
    while let Some(row) = trigger_rows.next().await.map_err(AppError::LibSQL)? {
        if let Ok(name) = row.get::<String>(0) {
            triggers.push(name.clone());
            if let Ok(sql) = row.get::<Option<String>>(1) {
                trigger_sqls.push(serde_json::json!({
                    "name": name,
                    "sql": sql,
                }));
            }
        }
    }
    tracing::info!("[SYNC-TEST] Found {} entries sync triggers: {:?}", triggers.len(), triggers);
    for ts in &trigger_sqls {
        tracing::info!("[SYNC-TEST] Trigger SQL: {}", ts);
    }
    
    // Get current outbox count
    let mut outbox_before = conn
        .query("SELECT COUNT(*) FROM _sync_outbox", libsql::params![])
        .await
        .map_err(AppError::LibSQL)?;
    let count_before: i64 = if let Some(row) = outbox_before.next().await.map_err(AppError::LibSQL)? {
        row.get(0).unwrap_or(0)
    } else {
        0
    };
    
    // Check _suppress_triggers value directly
    let mut suppress_check = conn
        .query("SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers'", libsql::params![])
        .await
        .map_err(AppError::LibSQL)?;
    let suppress_value: String = if let Some(row) = suppress_check.next().await.map_err(AppError::LibSQL)? {
        row.get::<String>(0).unwrap_or_else(|_| "0".to_string())
    } else {
        "0".to_string()
    };
    tracing::info!("[SYNC-TEST] _suppress_triggers value in DB: '{}'", suppress_value);
    
    // Update _updated_at to trigger the sync trigger
    let now_ms = chrono::Utc::now().timestamp_millis();
    tracing::info!("[SYNC-TEST] Executing: UPDATE entries SET _updated_at = {} WHERE id = '{}'", now_ms, entry_id_clone);
    conn.execute(
        "UPDATE entries SET _updated_at = ?1 WHERE id = ?2",
        libsql::params![now_ms, entry_id_clone.as_str()],
    )
    .await
    .map_err(AppError::LibSQL)?;
    
    tracing::info!("[SYNC-TEST] Updated entry _updated_at to {}", now_ms);
    
    // Immediately check if anything was inserted into outbox
    let mut immediate_check = conn
        .query("SELECT COUNT(*) FROM _sync_outbox", libsql::params![])
        .await
        .map_err(AppError::LibSQL)?;
    let immediate_count: i64 = if let Some(row) = immediate_check.next().await.map_err(AppError::LibSQL)? {
        row.get(0).unwrap_or(0)
    } else {
        0
    };
    tracing::info!("[SYNC-TEST] Immediate outbox count after update: {}", immediate_count);
    
    // Wait a tiny bit for trigger to fire
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Check outbox count after
    let mut outbox_after = conn
        .query("SELECT COUNT(*) FROM _sync_outbox", libsql::params![])
        .await
        .map_err(AppError::LibSQL)?;
    let count_after: i64 = if let Some(row) = outbox_after.next().await.map_err(AppError::LibSQL)? {
        row.get(0).unwrap_or(0)
    } else {
        0
    };
    
    tracing::info!("[SYNC-TEST] Outbox count before: {}, after: {}", count_before, count_after);
    
    // Check what's in the outbox after
    let mut outbox_items = conn
        .query("SELECT entity, entity_id, op FROM _sync_outbox", libsql::params![])
        .await
        .map_err(AppError::LibSQL)?;
    let mut outbox_entries = Vec::new();
    while let Some(row) = outbox_items.next().await.map_err(AppError::LibSQL)? {
        let entity: Option<String> = row.get(0).ok();
        let entity_id: Option<String> = row.get(1).ok();
        let op: Option<String> = row.get(2).ok();
        outbox_entries.push(serde_json::json!({
            "entity": entity,
            "entity_id": entity_id,
            "op": op,
        }));
    }
    
    // Try to manually insert into outbox to verify it works
    let test_entity_id = format!("test_{}", now_ms);
    let manual_insert_result = conn.execute(
        "INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at) VALUES ('entries', ?1, 'upsert', ?2)",
        libsql::params![test_entity_id.as_str(), now_ms],
    )
    .await;
    let manual_insert_works = manual_insert_result.is_ok();
    if manual_insert_works {
        // Clean up test entry
        let _ = conn.execute(
            "DELETE FROM _sync_outbox WHERE entity_id = ?1",
            libsql::params![test_entity_id.as_str()],
        )
        .await;
    }
    
    Ok(serde_json::json!({
        "entry_id": entry_id_clone,
        "sync_id": sync_id,
        "old_updated_at": old_updated_at,
        "new_updated_at": now_ms,
        "suppress_triggers_value": suppress_value,
        "triggers_found": triggers.len(),
        "trigger_names": triggers,
        "trigger_sqls": trigger_sqls,
        "outbox_count_before": count_before,
        "outbox_count_after": count_after,
        "trigger_fired": count_after > count_before,
        "outbox_entries": outbox_entries,
        "manual_insert_works": manual_insert_works,
    }))
}
