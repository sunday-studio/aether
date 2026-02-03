use crate::commands::params::{EmptyPathParams, EmptyQueryParams, EmptyRequest, IdPathParams, PaginationQueryParams};
use crate::db::models::Canvas;
use crate::db::{connection, DbState, CanvasRepository};
use crate::error::{AppError, Result};
use crate::handlers::common::PaginationResponse;
use crate::utils::{log_create, log_delete, log_update};
use serde::Deserialize;
use tauri::State;
use utoipa::ToSchema;

/// Request to create a canvas
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateCanvasRequest {
    pub name: String,
    #[serde(default)]
    pub canvas_data: Option<serde_json::Value>,
}

/// Request to update a canvas
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCanvasRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub canvas_data: Option<serde_json::Value>,
}

/// Get all canvases
#[utoipa::path(
    get,
    path = "/v1/canvas",
    tag = "Canvases",
    params(
        ("limit" = Option<u32>, Query, description = "Number of canvases per page (max 1000)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination")
    ),
    responses(
        (status = 200, description = "Paginated list of canvases", body = PaginatedCanvases),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_canvases(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<PaginationQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<PaginationResponse<Canvas>> {
    let params = query_params.unwrap_or_default();
    let repo = CanvasRepository::new(connection::get_database(&*state));
    let (canvases, next_cursor, has_more) = repo
        .find_all(params.normalize_limit(), params.cursor)
        .await?;
    Ok(PaginationResponse::new(canvases, next_cursor, has_more))
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
        (status = 200, description = "Canvas found", body = Canvas),
        (status = 404, description = "Canvas not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_canvas_by_id(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<Canvas> {
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
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
        (status = 200, description = "Created canvas", body = Canvas),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn create_canvas(
    state: State<'_, DbState>,
    request_data: Option<CreateCanvasRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<Canvas> {
    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    if request.name.is_empty() {
        return Err(AppError::BadRequest("Name is required".to_string()));
    }

    // Default empty canvas data if not provided (JSON Canvas format)
    let default_canvas_data = serde_json::json!({
        "nodes": [],
        "edges": []
    });
    let canvas_data = request.canvas_data.unwrap_or(default_canvas_data);

    let db = connection::get_database(&*state);
    let repo = CanvasRepository::new(db.clone());
    let canvas = repo.create(request.name, canvas_data).await?;
    
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
        (status = 200, description = "Updated canvas", body = Canvas),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Canvas not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn update_canvas(
    state: State<'_, DbState>,
    request_data: Option<UpdateCanvasRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<Canvas> {
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }

    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;

    let db = connection::get_database(&*state);
    let repo = CanvasRepository::new(db.clone());
    let canvas = repo.update(&id, request.name, request.canvas_data).await?;
    
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
pub async fn delete_canvas(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<()> {
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
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
