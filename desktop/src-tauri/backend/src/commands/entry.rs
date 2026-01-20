use crate::db::{connection, DbState, EntryRepository};
use crate::error::{AppError, Result};
use crate::handlers::entry::{CreateEntryRequest, UpdateEntryRequest};
use tauri::State;

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
#[tauri::command]
pub async fn get_entries(state: State<'_, DbState>) -> Result<Vec<crate::db::models::Entry>> {
    let repo = EntryRepository::new(connection::get_database(&*state));
    repo.find_all().await
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
#[tauri::command]
pub async fn get_entry_by_id(
    state: State<'_, DbState>,
    id: String,
) -> Result<crate::db::models::Entry> {
    let repo = EntryRepository::new(connection::get_database(&*state));
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Entry {} not found", id)))
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
#[tauri::command]
pub async fn create_entry(
    state: State<'_, DbState>,
    payload: CreateEntryRequest,
) -> Result<crate::db::models::Entry> {
    let repo = EntryRepository::new(connection::get_database(&*state));
    repo.create(
        payload.document,
        payload.date,
        payload.is_pinned.unwrap_or(false),
        payload.is_archived.unwrap_or(false),
        payload.is_deleted.unwrap_or(false),
    )
    .await
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
#[tauri::command]
pub async fn bulk_create_entries(
    state: State<'_, DbState>,
    payload: Vec<CreateEntryRequest>,
) -> Result<Vec<crate::db::models::Entry>> {
    let repo = EntryRepository::new(connection::get_database(&*state));
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
    repo.bulk_create(entries_data).await
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
#[tauri::command]
pub async fn update_entry(
    state: State<'_, DbState>,
    id: String,
    payload: UpdateEntryRequest,
) -> Result<crate::db::models::Entry> {
    let repo = EntryRepository::new(connection::get_database(&*state));
    repo.update(
        &id,
        payload.document,
        payload.is_pinned.unwrap_or(false),
        payload.is_archived.unwrap_or(false),
        payload.is_deleted.unwrap_or(false),
        payload.updated_at,
    )
    .await
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
#[tauri::command]
pub async fn delete_entry(state: State<'_, DbState>, id: String) -> Result<()> {
    let repo = EntryRepository::new(connection::get_database(&*state));
    repo.delete(&id).await
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
#[tauri::command]
pub async fn add_tags_to_entry(
    state: State<'_, DbState>,
    id: String,
    tag_ids: Vec<String>,
) -> Result<crate::db::models::Entry> {
    let repo = EntryRepository::new(connection::get_database(&*state));
    repo.add_tags(&id, tag_ids).await?;
    
    // Return updated entry
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Entry {} not found", id)))
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
#[tauri::command]
pub async fn remove_tags_from_entry(
    state: State<'_, DbState>,
    id: String,
    tag_id: String,
) -> Result<crate::db::models::Entry> {
    let repo = EntryRepository::new(connection::get_database(&*state));
    repo.remove_tags(&id, tag_id).await?;
    
    // Return updated entry
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Entry {} not found", id)))
}
