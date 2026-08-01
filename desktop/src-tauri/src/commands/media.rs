use crate::commands::params::{EmptyRequest, EntryIdPathParams};
use crate::db::connection;
use crate::db::repositories::MediaRepository;
use crate::error::{AppError, Result};
use tauri::State;

/// Get image and video media items for an entry.
#[utoipa::path(
    get,
    path = "/v1/entry/{entryId}/media",
    tag = "Media",
    params(
        ("entryId" = String, Path, description = "Entry ID")
    ),
    responses(
        (status = 200, description = "List of media items", body = Vec<MediaItem>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_media_items_for_entry(
    state: State<'_, crate::DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<crate::commands::params::EmptyQueryParams>,
    path_params: Option<EntryIdPathParams>,
) -> Result<Vec<crate::db::models::MediaItem>> {
    let _guard = connection::with_db_access(&*state).await;
    let entry_id = path_params
        .map(|p| p.entry_id)
        .ok_or_else(|| AppError::BadRequest("Entry ID is required".to_string()))?;
    if entry_id.is_empty() {
        return Err(AppError::BadRequest("Entry ID is required".to_string()));
    }

    let database = connection::get_database(&*state);
    let repo = MediaRepository::new(database);
    let items = repo.find_by_entry_id(&entry_id).await?;
    Ok(items
        .into_iter()
        .filter(|item| matches!(item.media_type.as_str(), "image" | "video"))
        .collect())
}
