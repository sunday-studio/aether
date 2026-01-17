use crate::db::{connection, DbState};
use crate::error::{AppError, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;
use utoipa::ToSchema;

/// Manually trigger sync with remote database
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
