use crate::db::connection;
use crate::settings;
use crate::sync::{self, SyncEngine, SyncStatus};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn configure_sync(
    app: AppHandle,
    engine: State<'_, Arc<SyncEngine>>,
    server_url: String,
    passphrase: String,
) -> Result<(), String> {
    engine
        .configure(server_url, passphrase)
        .await
        .map_err(|e| e.to_string())?;
    if let Ok(status) = engine.status().await {
        let _ = app.emit("sync-status", &status);
    }
    Ok(())
}

#[tauri::command]
pub async fn sync_now(app: AppHandle, engine: State<'_, Arc<SyncEngine>>) -> Result<SyncStatus, String> {
    let status = engine.sync().await.map_err(|e| e.to_string())?;
    let _ = app.emit("sync-status", &status);
    Ok(status)
}

#[tauri::command]
pub async fn get_sync_status(engine: State<'_, Arc<SyncEngine>>) -> Result<SyncStatus, String> {
    engine.status().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn disconnect_sync(engine: State<'_, Arc<SyncEngine>>) -> Result<(), String> {
    engine.disconnect().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reconnect_sync(
    app: AppHandle,
    engine: State<'_, Arc<SyncEngine>>,
    passphrase: String,
) -> Result<SyncStatus, String> {
    engine.reconnect(passphrase).await.map_err(|e| e.to_string())?;
    let status = engine.status().await.map_err(|e| e.to_string())?;
    let _ = app.emit("sync-status", &status);
    Ok(status)
}

/// For on-demand media: ensure the blob is on disk before the frontend reads/plays it.
/// No-op when sync.media_sync_policy is "auto" or sync is not configured.
#[tauri::command]
pub async fn ensure_media_blob(
    db: State<'_, crate::DbState>,
    engine: State<'_, Arc<SyncEngine>>,
    media_id: String,
) -> Result<(), String> {
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
    .map_err(|e| e.to_string())
}
