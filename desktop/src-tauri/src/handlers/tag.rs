use crate::db::{connection, DbState, TagRepository};
use crate::error::{AppError, Result};
use crate::utils::log_create;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateTagRequest {
    pub name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct BulkCreateTagsRequest {
    pub tags: Vec<CreateTagRequest>,
}

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
pub async fn get_all_tags(State(state): State<DbState>) -> Result<impl IntoResponse> {
    let repo = TagRepository::new(connection::get_database(&state));
    let tags = repo.find_all().await?;
    Ok((StatusCode::OK, Json(tags)))
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
pub async fn create_tag(
    State(state): State<DbState>,
    Json(payload): Json<CreateTagRequest>,
) -> Result<impl IntoResponse> {
    if payload.name.is_empty() {
        return Err(AppError::BadRequest("Tag name cannot be empty".to_string()));
    }

    let db = connection::get_database(&state);
    let repo = TagRepository::new(db.clone());
    let tag = repo.create(payload.name).await?;
    
    // Log activity
    let _ = log_create(db, "tag".to_string(), tag.id.clone()).await;
    
    Ok((StatusCode::OK, Json(tag)))
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
pub async fn bulk_create_tags(
    State(state): State<DbState>,
    Json(payload): Json<Vec<CreateTagRequest>>,
) -> Result<impl IntoResponse> {
    let db = connection::get_database(&state);
    let repo = TagRepository::new(db.clone());
    let names: Vec<String> = payload.into_iter().map(|t| t.name).collect();
    let tags = repo.bulk_create(names).await?;
    
    // Log activities for each created tag
    for tag in &tags {
        let _ = log_create(db.clone(), "tag".to_string(), tag.id.clone()).await;
    }
    
    Ok((StatusCode::OK, Json(tags)))
}
