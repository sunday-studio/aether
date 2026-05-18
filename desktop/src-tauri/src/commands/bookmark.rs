use crate::commands::params::{
    BookmarkQueryParams, EmptyPathParams, EmptyQueryParams, EmptyRequest,
    ExtractMetadataQueryParams, IdPathParams,
};
use crate::db::models::Bookmark;
use crate::db::{connection, BookmarkRepository, DbState, SearchDocumentRepository};
use crate::error::{AppError, Result};
use crate::utils::metadata::extractor::ExtractedMetadata;
use crate::utils::metadata::MetadataExtractor;
use crate::utils::{log_create, log_delete, log_tag_operation, log_update};
use serde::Deserialize;
use tauri::State;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateBookmarkRequest {
    pub url: String,
    #[serde(default)]
    pub tag_ids: Option<Vec<String>>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBookmarkRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub favicon_url: Option<String>,
    #[serde(default)]
    pub site_name: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub metadata_json: Option<serde_json::Value>,
    #[serde(default)]
    pub is_archived: Option<bool>,
}

/// Request to add tags to a bookmark
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct AddTagsToBookmarkRequest {
    pub tag_ids: Vec<String>,
}

/// Request to remove tags from a bookmark
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RemoveTagsFromBookmarkRequest {
    pub tag_ids: Vec<String>,
}

/// Get all bookmarks
#[utoipa::path(
    get,
    path = "/v1/bookmarks",
    tag = "Bookmarks",
    params(
        ("is_archived" = Option<bool>, Query, description = "Filter by archived status"),
        ("tag_ids" = Option<Vec<String>>, Query, description = "Filter by tag IDs"),
        ("content_type" = Option<String>, Query, description = "Filter by content type"),
        ("limit" = Option<u32>, Query, description = "Number of bookmarks per page (max 1000)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination")
    ),
    responses(
        (status = 200, description = "Paginated list of bookmarks", body = PaginatedBookmarks),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_bookmarks(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<BookmarkQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<crate::commands::common::PaginationResponse<Bookmark>> {
    let _guard = connection::with_db_access(&*state).await;
    let params = query_params.unwrap_or_default();
    let repo = BookmarkRepository::new(connection::get_database(&*state));
    let (bookmarks, next_cursor, has_more) = repo
        .find_all(
            params.is_archived,
            params.tag_ids,
            params.content_type,
            params.limit.map(|l| l.min(1000)),
            params.cursor,
        )
        .await?;
    Ok(crate::commands::common::PaginationResponse::new(
        bookmarks,
        next_cursor,
        has_more,
    ))
}

/// Get bookmark by ID
#[utoipa::path(
    get,
    path = "/v1/bookmarks/{id}",
    tag = "Bookmarks",
    params(
        ("id" = String, Path, description = "Bookmark ID")
    ),
    responses(
        (status = 200, description = "Bookmark found", body = Bookmark),
        (status = 404, description = "Bookmark not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_bookmark_by_id(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<crate::db::models::Bookmark> {
    let _guard = connection::with_db_access(&*state).await;
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }
    let repo = BookmarkRepository::new(connection::get_database(&*state));
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Bookmark {} not found", id)))
}

/// Create a new bookmark
#[utoipa::path(
    post,
    path = "/v1/bookmarks",
    tag = "Bookmarks",
    request_body = CreateBookmarkRequest,
    responses(
        (status = 200, description = "Created bookmark", body = Bookmark),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn create_bookmark(
    state: State<'_, DbState>,
    request_data: Option<CreateBookmarkRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<crate::db::models::Bookmark> {
    let _guard = connection::with_db_access(&*state).await;
    let request =
        request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    if request.url.is_empty() {
        return Err(AppError::BadRequest("URL is required".to_string()));
    }

    // Extract metadata
    let metadata = MetadataExtractor::extract(&request.url).await?;

    let db = connection::get_database(&*state);
    let repo = BookmarkRepository::new(db.clone());

    let bookmark = repo
        .create(
            request.url,
            metadata.title,
            metadata.description,
            metadata.image_url,
            metadata.favicon_url,
            metadata.site_name,
            metadata.author,
            metadata.published_at,
            metadata.content_type,
            metadata.metadata_json,
        )
        .await?;

    // Add tags if provided
    if let Some(tag_ids) = request.tag_ids {
        if !tag_ids.is_empty() {
            repo.add_tags(&bookmark.id, tag_ids).await?;
        }
    }

    if let Err(e) = SearchDocumentRepository::new(db.clone())
        .reindex_resource("bookmark", &bookmark.id)
        .await
    {
        tracing::warn!(
            "Failed to reindex bookmark {} for search: {}",
            bookmark.id,
            e
        );
    }

    // Log activity
    if let Err(e) = log_create(db.clone(), "bookmark".to_string(), bookmark.id.clone()).await {
        tracing::warn!("Failed to log bookmark creation activity: {}", e);
    }

    Ok(bookmark)
}

/// Update a bookmark
#[utoipa::path(
    put,
    path = "/v1/bookmarks/{id}",
    tag = "Bookmarks",
    params(
        ("id" = String, Path, description = "Bookmark ID")
    ),
    request_body = UpdateBookmarkRequest,
    responses(
        (status = 200, description = "Updated bookmark", body = Bookmark),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Bookmark not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn update_bookmark(
    state: State<'_, DbState>,
    request_data: Option<UpdateBookmarkRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<crate::db::models::Bookmark> {
    let _guard = connection::with_db_access(&*state).await;
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }

    let request =
        request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;

    let db = connection::get_database(&*state);
    let repo = BookmarkRepository::new(db.clone());
    let bookmark = repo
        .update(
            &id,
            request.title,
            request.description,
            request.image_url,
            request.favicon_url,
            request.site_name,
            request.author,
            request.published_at,
            request.content_type,
            request.metadata_json,
            request.is_archived,
        )
        .await?;

    if let Err(e) = SearchDocumentRepository::new(db.clone())
        .reindex_resource("bookmark", &bookmark.id)
        .await
    {
        tracing::warn!(
            "Failed to reindex bookmark {} for search: {}",
            bookmark.id,
            e
        );
    }

    // Log activity
    if let Err(e) = log_update(db.clone(), "bookmark".to_string(), bookmark.id.clone()).await {
        tracing::warn!("Failed to log bookmark update activity: {}", e);
    }

    Ok(bookmark)
}

/// Delete a bookmark
#[utoipa::path(
    delete,
    path = "/v1/bookmarks/{id}",
    tag = "Bookmarks",
    params(
        ("id" = String, Path, description = "Bookmark ID")
    ),
    responses(
        (status = 204, description = "Bookmark deleted"),
        (status = 404, description = "Bookmark not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn delete_bookmark(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<()> {
    let _guard = connection::with_db_access(&*state).await;
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }
    let db = connection::get_database(&*state);
    let repo = BookmarkRepository::new(db.clone());
    repo.delete(&id).await?;

    if let Err(e) = SearchDocumentRepository::new(db.clone())
        .reindex_resource("bookmark", &id)
        .await
    {
        tracing::warn!("Failed to remove bookmark {} from search index: {}", id, e);
    }

    // Log activity
    if let Err(e) = log_delete(db.clone(), "bookmark".to_string(), id.clone()).await {
        tracing::warn!(
            "Failed to log bookmark deletion activity for bookmark {}: {}",
            id,
            e
        );
    }

    Ok(())
}

/// Add tags to a bookmark
#[utoipa::path(
    post,
    path = "/v1/bookmarks/{id}/tags",
    tag = "Bookmarks",
    params(
        ("id" = String, Path, description = "Bookmark ID")
    ),
    request_body = Vec<String>,
    responses(
        (status = 200, description = "Tags added to bookmark", body = Bookmark),
        (status = 404, description = "Bookmark or tag not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn add_tags_to_bookmark(
    state: State<'_, DbState>,
    request_data: Option<AddTagsToBookmarkRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<crate::db::models::Bookmark> {
    let _guard = connection::with_db_access(&*state).await;
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }
    let request =
        request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    let db = connection::get_database(&*state);
    let repo = BookmarkRepository::new(db.clone());
    repo.add_tags(&id, request.tag_ids).await?;

    // Log activity
    if let Err(e) =
        log_tag_operation(db.clone(), "add_tags", "bookmark".to_string(), id.clone()).await
    {
        tracing::warn!("Failed to log add_tags activity for bookmark {}: {}", id, e);
    }

    // Return updated bookmark
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Bookmark {} not found", id)))
}

/// Remove tags from a bookmark
#[utoipa::path(
    delete,
    path = "/v1/bookmarks/{id}/tags",
    tag = "Bookmarks",
    params(
        ("id" = String, Path, description = "Bookmark ID")
    ),
    request_body = Vec<String>,
    responses(
        (status = 200, description = "Tags removed from bookmark", body = Bookmark),
        (status = 404, description = "Bookmark or tag not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn remove_tags_from_bookmark(
    state: State<'_, DbState>,
    request_data: Option<RemoveTagsFromBookmarkRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<crate::db::models::Bookmark> {
    let _guard = connection::with_db_access(&*state).await;
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }
    let request =
        request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    let db = connection::get_database(&*state);
    let repo = BookmarkRepository::new(db.clone());
    repo.remove_tags(&id, request.tag_ids).await?;

    // Log activity
    if let Err(e) = log_tag_operation(
        db.clone(),
        "remove_tags",
        "bookmark".to_string(),
        id.clone(),
    )
    .await
    {
        tracing::warn!(
            "Failed to log remove_tags activity for bookmark {}: {}",
            id,
            e
        );
    }

    // Return updated bookmark
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Bookmark {} not found", id)))
}

/// Extract metadata from a URL without saving
#[utoipa::path(
    get,
    path = "/v1/bookmarks/extract-metadata",
    tag = "Bookmarks",
    params(
        ("url" = String, Query, description = "URL to extract metadata from")
    ),
    responses(
        (status = 200, description = "Extracted metadata", body = ExtractedMetadata),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn extract_metadata(
    _request_data: Option<EmptyRequest>,
    query_params: Option<ExtractMetadataQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<ExtractedMetadata> {
    let params = query_params
        .ok_or_else(|| AppError::BadRequest("Query parameters are required".to_string()))?;
    if params.url.is_empty() {
        return Err(AppError::BadRequest("URL is required".to_string()));
    }
    MetadataExtractor::extract(&params.url).await
}
