use crate::db::{connection, DbState, TagRepository};
use crate::error::{AppError, Result};
use crate::handlers::tag::CreateTagRequest;
use crate::utils::log_create;
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
#[tauri::command(rename_all = "camelCase")]
pub async fn create_tag(
    state: State<'_, DbState>,
    name: String,
) -> Result<crate::db::models::Tag> {
    if name.is_empty() {
        return Err(AppError::BadRequest("Tag name cannot be empty".to_string()));
    }

    let db = connection::get_database(&*state);
    let repo = TagRepository::new(db.clone());
    let tag = repo.create(name).await?;
    
    // Log activity
    if let Err(e) = log_create(db, "tag".to_string(), tag.id.clone()).await {
        tracing::warn!("Failed to log tag creation activity: {}", e);
    }
    
    Ok(tag)
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
    let db = connection::get_database(&*state);
    let repo = TagRepository::new(db.clone());
    let names: Vec<String> = payload.into_iter().map(|t| t.name).collect();
    let tags = repo.bulk_create(names).await?;
    
    // Log activities for each created tag
    for tag in &tags {
        if let Err(e) = log_create(db.clone(), "tag".to_string(), tag.id.clone()).await {
            tracing::warn!("Failed to log tag creation activity for tag {}: {}", tag.id, e);
        }
    }
    
    Ok(tags)
}
