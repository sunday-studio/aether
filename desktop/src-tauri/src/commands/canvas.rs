use crate::db::{connection, DbState, CanvasRepository};
use crate::error::{AppError, Result};
use crate::utils::{log_create, log_delete, log_update};
use tauri::State;

/// Get all canvases
#[utoipa::path(
    get,
    path = "/v1/canvas",
    tag = "Canvases",
    responses(
        (status = 200, description = "List of all canvases", body = Vec<crate::db::models::Canvas>),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_canvases(state: State<'_, DbState>) -> Result<Vec<crate::db::models::Canvas>> {
    let repo = CanvasRepository::new(connection::get_database(&*state));
    repo.find_all().await
}

/// Get canvas by ID
#[utoipa::path(
    get,
    path = "/v1/canvas/{id}",
    tag = "Canvases",
    params(
        ("id" = String, Path, description = "Canvas ID")
    ),
    responses(
        (status = 200, description = "Canvas found", body = crate::db::models::Canvas),
        (status = 404, description = "Canvas not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_canvas_by_id(
    state: State<'_, DbState>,
    id: String,
) -> Result<crate::db::models::Canvas> {
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }
    let repo = CanvasRepository::new(connection::get_database(&*state));
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Canvas {} not found", id)))
}

/// Create a new canvas
#[utoipa::path(
    post,
    path = "/v1/canvas",
    tag = "Canvases",
    request_body = CreateCanvasRequest,
    responses(
        (status = 200, description = "Created canvas", body = crate::db::models::Canvas),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command(rename_all = "camelCase")]
pub async fn create_canvas(
    state: State<'_, DbState>,
    name: String,
    canvas_data: Option<serde_json::Value>,
) -> Result<crate::db::models::Canvas> {
    if name.is_empty() {
        return Err(AppError::BadRequest("Name is required".to_string()));
    }

    // Default empty canvas data if not provided (JSON Canvas format)
    let default_canvas_data = serde_json::json!({
        "nodes": [],
        "edges": []
    });
    let canvas_data = canvas_data.unwrap_or(default_canvas_data);

    let db = connection::get_database(&*state);
    let repo = CanvasRepository::new(db.clone());
    let canvas = repo.create(name, canvas_data).await?;
    
    // Log activity
    if let Err(e) = log_create(db, "canvas".to_string(), canvas.id.clone()).await {
        tracing::warn!("Failed to log canvas creation activity: {}", e);
    }
    
    Ok(canvas)
}

/// Update a canvas
#[utoipa::path(
    put,
    path = "/v1/canvas/{id}",
    tag = "Canvases",
    params(
        ("id" = String, Path, description = "Canvas ID")
    ),
    request_body = UpdateCanvasRequest,
    responses(
        (status = 200, description = "Updated canvas", body = crate::db::models::Canvas),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Canvas not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command(rename_all = "camelCase")]
pub async fn update_canvas(
    state: State<'_, DbState>,
    id: String,
    name: Option<String>,
    canvas_data: Option<serde_json::Value>,
) -> Result<crate::db::models::Canvas> {
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }

    let db = connection::get_database(&*state);
    let repo = CanvasRepository::new(db.clone());
    let canvas = repo.update(&id, name, canvas_data).await?;
    
    // Log activity
    if let Err(e) = log_update(db, "canvas".to_string(), canvas.id.clone()).await {
        tracing::warn!("Failed to log canvas update activity: {}", e);
    }
    
    Ok(canvas)
}

/// Delete a canvas
#[utoipa::path(
    delete,
    path = "/v1/canvas/{id}",
    tag = "Canvases",
    params(
        ("id" = String, Path, description = "Canvas ID")
    ),
    responses(
        (status = 204, description = "Canvas deleted"),
        (status = 404, description = "Canvas not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn delete_canvas(state: State<'_, DbState>, id: String) -> Result<()> {
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }
    let db = connection::get_database(&*state);
    let repo = CanvasRepository::new(db.clone());
    repo.delete(&id).await?;
    
    // Log activity
    if let Err(e) = log_delete(db, "canvas".to_string(), id.clone()).await {
        tracing::warn!("Failed to log canvas deletion activity for canvas {}: {}", id, e);
    }
    
    Ok(())
}
