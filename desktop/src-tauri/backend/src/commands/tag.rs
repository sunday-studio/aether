use crate::db::{connection, DbState, TagRepository};
use crate::error::{AppError, Result};
use crate::handlers::tag::CreateTagRequest;
use tauri::State;

/// Get all tags
#[utoipa::path(
    get,
    path = "/v1/tags",
    tag = "Tags",
    responses(
        (status = 200, description = "List of all tags", body = Vec<crate::db::models::Tag>),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_all_tags(state: State<'_, DbState>) -> Result<Vec<crate::db::models::Tag>> {
    let repo = TagRepository::new(connection::get_database(&*state));
    repo.find_all().await
}

/// Create a new tag
#[utoipa::path(
    post,
    path = "/v1/tags",
    tag = "Tags",
    request_body = CreateTagRequest,
    responses(
        (status = 200, description = "Created tag", body = crate::db::models::Tag),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn create_tag(
    state: State<'_, DbState>,
    payload: CreateTagRequest,
) -> Result<crate::db::models::Tag> {
    if payload.name.is_empty() {
        return Err(AppError::BadRequest("Tag name cannot be empty".to_string()));
    }

    let repo = TagRepository::new(connection::get_database(&*state));
    repo.create(payload.name).await
}

/// Bulk create tags
#[utoipa::path(
    post,
    path = "/v1/tags/bulk-create",
    tag = "Tags",
    request_body = Vec<CreateTagRequest>,
    responses(
        (status = 200, description = "Created tags", body = Vec<crate::db::models::Tag>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn bulk_create_tags(
    state: State<'_, DbState>,
    payload: Vec<CreateTagRequest>,
) -> Result<Vec<crate::db::models::Tag>> {
    let repo = TagRepository::new(connection::get_database(&*state));
    let names: Vec<String> = payload.into_iter().map(|t| t.name).collect();
    repo.bulk_create(names).await
}
