use crate::commands::params::{EmptyPathParams, EmptyQueryParams, EmptyRequest, PaginationQueryParams};
use crate::db::models::Tag;
use crate::db::{connection, DbState, TagRepository};
use crate::error::{AppError, Result};
use crate::commands::common::PaginationResponse;
use crate::utils::log_create;
use serde::Deserialize;
use tauri::State;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateTagRequest {
    pub name: String,
}

/// Get all tags
#[utoipa::path(
    get,
    path = "/v1/tags",
    tag = "Tags",
    params(
        ("limit" = Option<u32>, Query, description = "Number of tags per page (max 1000)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination")
    ),
    responses(
        (status = 200, description = "Paginated list of tags", body = PaginatedTags),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_all_tags(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<PaginationQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<PaginationResponse<Tag>> {
    let _guard = connection::with_db_access(&*state).await;
    let params = query_params.unwrap_or_default();
    let repo = TagRepository::new(connection::get_database(&*state));
    let (tags, next_cursor, has_more) = repo
        .find_all(params.normalize_limit(), params.cursor)
        .await?;
    Ok(PaginationResponse::new(tags, next_cursor, has_more))
}

/// Create a new tag
#[utoipa::path(
    post,
    path = "/v1/tags",
    tag = "Tags",
    request_body = CreateTagRequest,
    responses(
        (status = 200, description = "Created tag", body = Tag),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn create_tag(
    state: State<'_, DbState>,
    request_data: Option<CreateTagRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<Tag> {
    let _guard = connection::with_db_access(&*state).await;
    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    if request.name.is_empty() {
        return Err(AppError::BadRequest("Tag name cannot be empty".to_string()));
    }

    let db = connection::get_database(&*state);
    let repo = TagRepository::new(db.clone());
    let tag = repo.create(request.name).await?;
    
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
        (status = 200, description = "Created tags", body = Vec<Tag>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn bulk_create_tags(
    state: State<'_, DbState>,
    request_data: Option<Vec<CreateTagRequest>>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<Vec<Tag>> {
    let _guard = connection::with_db_access(&*state).await;
    let payload = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
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
