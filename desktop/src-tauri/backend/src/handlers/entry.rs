use crate::db::{connection, DbState, EntryRepository};
use crate::error::{AppError, Result};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateEntryRequest {
    pub document: String,
    #[serde(default = "default_created_at")]
    pub date: chrono::DateTime<Utc>,
    #[serde(default)]
    pub is_pinned: Option<bool>,
    #[serde(default)]
    pub is_archived: Option<bool>,
    #[serde(default)]
    pub is_deleted: Option<bool>,
}

fn default_created_at() -> chrono::DateTime<Utc> {
    Utc::now()
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateEntryRequest {
    pub document: String,
    #[serde(default)]
    pub is_pinned: Option<bool>,
    #[serde(default)]
    pub is_archived: Option<bool>,
    #[serde(default)]
    pub is_deleted: Option<bool>,
    #[serde(default)]
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

/// Get all entries
#[utoipa::path(
    get,
    path = "/v1/entry",
    tag = "Entries",
    responses(
        (status = 200, description = "List of all entries", body = Vec<crate::db::models::Entry>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_entries(State(state): State<DbState>) -> Result<impl IntoResponse> {
    let repo = EntryRepository::new(connection::get_database(&state));
    let entries = repo.find_all().await?;
    Ok((StatusCode::OK, Json(entries)))
}

/// Get entry by ID
#[utoipa::path(
    get,
    path = "/v1/entry/{id}",
    tag = "Entries",
    params(
        ("id" = String, Path, description = "Entry ID")
    ),
    responses(
        (status = 200, description = "Entry found", body = crate::db::models::Entry),
        (status = 404, description = "Entry not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_entry_by_id(
    State(state): State<DbState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let repo = EntryRepository::new(connection::get_database(&state));
    match repo.find_by_id(&id).await? {
        Some(entry) => Ok((StatusCode::OK, Json(entry))),
        None => Err(AppError::NotFound(format!("Entry {} not found", id))),
    }
}

/// Create a new entry
#[utoipa::path(
    post,
    path = "/v1/entry",
    tag = "Entries",
    request_body = CreateEntryRequest,
    responses(
        (status = 200, description = "Created entry", body = crate::db::models::Entry),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_entry(
    State(state): State<DbState>,
    Json(payload): Json<CreateEntryRequest>,
) -> Result<impl IntoResponse> {
    let repo = EntryRepository::new(connection::get_database(&state));
    let entry = repo
        .create(
            payload.document,
            payload.date,
            payload.is_pinned.unwrap_or(false),
            payload.is_archived.unwrap_or(false),
            payload.is_deleted.unwrap_or(false),
        )
        .await?;
    Ok((StatusCode::OK, Json(entry)))
}

/// Bulk create entries
#[utoipa::path(
    post,
    path = "/v1/entry/bulk-create",
    tag = "Entries",
    request_body = Vec<CreateEntryRequest>,
    responses(
        (status = 200, description = "Created entries", body = Vec<crate::db::models::Entry>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn bulk_create_entries(
    State(state): State<DbState>,
    Json(payload): Json<Vec<CreateEntryRequest>>,
) -> Result<impl IntoResponse> {
    let repo = EntryRepository::new(connection::get_database(&state));
    let entries_data: Vec<_> = payload
        .into_iter()
        .map(|e| {
            (
                e.document,
                e.date,
                e.is_pinned.unwrap_or(false),
                e.is_archived.unwrap_or(false),
                e.is_deleted.unwrap_or(false),
            )
        })
        .collect();
    let entries = repo.bulk_create(entries_data).await?;
    Ok((StatusCode::OK, Json(entries)))
}

/// Update an entry
#[utoipa::path(
    put,
    path = "/v1/entry/{id}",
    tag = "Entries",
    params(
        ("id" = String, Path, description = "Entry ID")
    ),
    request_body = UpdateEntryRequest,
    responses(
        (status = 200, description = "Updated entry", body = crate::db::models::Entry),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Entry not found"),
        (status = 409, description = "Conflict: Record was modified by another device"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_entry(
    State(state): State<DbState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateEntryRequest>,
) -> Result<impl IntoResponse> {
    let repo = EntryRepository::new(connection::get_database(&state));
    let entry = repo
        .update(
            &id,
            payload.document,
            payload.is_pinned.unwrap_or(false),
            payload.is_archived.unwrap_or(false),
            payload.is_deleted.unwrap_or(false),
            payload.updated_at,
        )
        .await?;
    Ok((StatusCode::OK, Json(entry)))
}

/// Delete an entry
#[utoipa::path(
    delete,
    path = "/v1/entry/{id}",
    tag = "Entries",
    params(
        ("id" = String, Path, description = "Entry ID")
    ),
    responses(
        (status = 204, description = "Entry deleted"),
        (status = 404, description = "Entry not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_entry(
    State(state): State<DbState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let repo = EntryRepository::new(connection::get_database(&state));
    repo.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Add tags to an entry
#[utoipa::path(
    post,
    path = "/v1/entry/{id}/tags",
    tag = "Entries",
    params(
        ("id" = String, Path, description = "Entry ID")
    ),
    request_body = Vec<String>,
    responses(
        (status = 200, description = "Tags added to entry", body = crate::db::models::Entry),
        (status = 404, description = "Entry or tag not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn add_tags_to_entry(
    State(state): State<DbState>,
    Path(id): Path<String>,
    Json(tag_ids): Json<Vec<String>>,
) -> Result<impl IntoResponse> {
    let repo = EntryRepository::new(connection::get_database(&state));
    repo.add_tags(&id, tag_ids).await?;
    
    // Return updated entry
    let entry = repo.find_by_id(&id).await?
        .ok_or_else(|| AppError::NotFound(format!("Entry {} not found", id)))?;
    Ok((StatusCode::OK, Json(entry)))
}

/// Remove tags from an entry
#[utoipa::path(
    delete,
    path = "/v1/entry/{id}/tags",
    tag = "Entries",
    params(
        ("id" = String, Path, description = "Entry ID")
    ),
    request_body = String,
    responses(
        (status = 200, description = "Tag removed from entry", body = crate::db::models::Entry),
        (status = 404, description = "Entry or tag not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn remove_tags_from_entry(
    State(state): State<DbState>,
    Path(id): Path<String>,
    Json(tag_id): Json<String>,
) -> Result<impl IntoResponse> {
    let repo = EntryRepository::new(connection::get_database(&state));
    repo.remove_tags(&id, tag_id).await?;
    
    // Return updated entry
    let entry = repo.find_by_id(&id).await?
        .ok_or_else(|| AppError::NotFound(format!("Entry {} not found", id)))?;
    Ok((StatusCode::OK, Json(entry)))
}
