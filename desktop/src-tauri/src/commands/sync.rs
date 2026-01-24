use crate::sync::{SyncEngine, SyncStatus};
use tauri::State;

#[tauri::command]
pub async fn configure_sync(
    engine: State<'_, SyncEngine>,
    server_url: String,
    passphrase: String,
) -> Result<(), String> {
    engine
        .configure(server_url, passphrase)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_now(engine: State<'_, SyncEngine>) -> Result<SyncStatus, String> {
    engine.sync().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_sync_status(engine: State<'_, SyncEngine>) -> Result<SyncStatus, String> {
    engine.status().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn disconnect_sync(engine: State<'_, SyncEngine>) -> Result<(), String> {
    engine.disconnect().await.map_err(|e| e.to_string())
}
