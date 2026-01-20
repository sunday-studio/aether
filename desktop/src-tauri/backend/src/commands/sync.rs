use crate::db::{connection, DbState};
use crate::error::Result;
use serde_json::json;
use tauri::State;

/// Configure sync with remote database
/// 
/// This enables embedded replica mode with offline writes:
/// - Writes go to local WAL first (offline writes)
/// - Changes are automatically synced to remote
/// - Existing local data is preserved and synced
/// 
/// Reference: https://turso.tech/blog/introducing-offline-writes-for-turso
#[utoipa::path(
    post,
    path = "/v1/sync/configure",
    tag = "Sync",
    request_body = ConfigureSyncRequest,
    responses(
        (status = 200, description = "Sync configured successfully"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Failed to configure sync")
    )
)]
#[tauri::command(rename_all = "camelCase")]
pub async fn configure_sync(
    state: State<'_, DbState>,
    sync_url: String,
    auth_token: Option<String>,
) -> Result<serde_json::Value> {
    tracing::info!("Configuring sync with URL: {}", sync_url);
    
    connection::configure_sync(&*state, sync_url, auth_token).await?;

    Ok(json!({
        "success": true,
        "message": "Sync configured successfully"
    }))
}

/// Manually trigger sync with remote database
/// 
/// According to Turso's Offline Writes:
/// - Pushes local WAL frames (offline writes) to remote
/// - Pulls remote WAL frames to local
/// - Returns number of frames synced
/// 
/// Reference: https://turso.tech/blog/introducing-offline-writes-for-turso
#[utoipa::path(
    post,
    path = "/v1/sync",
    tag = "Sync",
    responses(
        (status = 200, description = "Sync completed successfully"),
        (status = 400, description = "Sync not available"),
        (status = 500, description = "Sync failed")
    )
)]
#[tauri::command]
pub async fn sync(state: State<'_, DbState>) -> Result<serde_json::Value> {
    tracing::info!("Manual sync triggered");
    
    let frames_synced = connection::sync_now(&*state).await?;

    tracing::info!("Manual sync completed, frames synced: {}", frames_synced);

    Ok(json!({
        "success": true,
        "framesSynced": frames_synced,
        "message": "Sync completed successfully"
    }))
}
