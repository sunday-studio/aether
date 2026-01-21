use crate::db::{connection, DbState};
use crate::error::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

/// Request to configure sync with remote database
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConfigureSyncRequest {
    pub sync_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
}

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
pub async fn configure_sync(
    State(state): State<DbState>,
    Json(payload): Json<ConfigureSyncRequest>,
) -> Result<impl IntoResponse> {
    tracing::info!("Configuring sync with URL: {}", payload.sync_url);
    
    connection::configure_sync(&state, payload.sync_url, payload.auth_token).await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "message": "Sync configured successfully"
        })),
    ))
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
pub async fn sync(State(state): State<DbState>) -> Result<impl IntoResponse> {
    tracing::info!("Manual sync triggered");
    
    let frames_synced = connection::sync_now(&state).await?;

    tracing::info!("Manual sync completed, frames synced: {}", frames_synced);

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "framesSynced": frames_synced,
            "message": "Sync completed successfully"
        })),
    ))
}
