use crate::db::{connection, DbState, BookmarkRepository};
use crate::error::{AppError, Result};
use crate::utils::{log_create, log_delete, log_tag_operation, log_update};
use crate::utils::metadata::MetadataExtractor;
use tauri::State;

/// Get all bookmarks
#[utoipa::path(
    get,
    path = "/v1/bookmarks",
    tag = "Bookmarks",
    responses(
        (status = 200, description = "List of all bookmarks", body = Vec<crate::db::models::Bookmark>),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_bookmarks(
    state: State<'_, DbState>,
    is_archived: Option<bool>,
    tag_ids: Option<Vec<String>>,
    content_type: Option<String>,
) -> Result<Vec<crate::db::models::Bookmark>> {
    let repo = BookmarkRepository::new(connection::get_database(&*state));
    repo.find_all(is_archived, tag_ids, content_type).await
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
        (status = 200, description = "Bookmark found", body = crate::db::models::Bookmark),
        (status = 404, description = "Bookmark not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_bookmark_by_id(
    state: State<'_, DbState>,
    id: String,
) -> Result<crate::db::models::Bookmark> {
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
        (status = 200, description = "Created bookmark", body = crate::db::models::Bookmark),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command(rename_all = "camelCase")]
pub async fn create_bookmark(
    state: State<'_, DbState>,
    url: String,
    tag_ids: Option<Vec<String>>,
) -> Result<crate::db::models::Bookmark> {
    if url.is_empty() {
        return Err(AppError::BadRequest("URL is required".to_string()));
    }

    // Extract metadata
    let metadata = MetadataExtractor::extract(&url).await?;

    let db = connection::get_database(&*state);
    let repo = BookmarkRepository::new(db.clone());
    
    let bookmark = repo.create(
        url,
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
    if let Some(tag_ids) = tag_ids {
        if !tag_ids.is_empty() {
            repo.add_tags(&bookmark.id, tag_ids).await?;
        }
    }

    // Log activity
    if let Err(e) = log_create(db, "bookmark".to_string(), bookmark.id.clone()).await {
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
        (status = 200, description = "Updated bookmark", body = crate::db::models::Bookmark),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Bookmark not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command(rename_all = "camelCase")]
pub async fn update_bookmark(
    state: State<'_, DbState>,
    id: String,
    title: Option<String>,
    description: Option<String>,
    image_url: Option<String>,
    favicon_url: Option<String>,
    site_name: Option<String>,
    author: Option<String>,
    published_at: Option<chrono::DateTime<chrono::Utc>>,
    content_type: Option<String>,
    metadata_json: Option<serde_json::Value>,
    is_archived: Option<bool>,
) -> Result<crate::db::models::Bookmark> {
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }

    let db = connection::get_database(&*state);
    let repo = BookmarkRepository::new(db.clone());
    let bookmark = repo.update(
        &id,
        title,
        description,
        image_url,
        favicon_url,
        site_name,
        author,
        published_at,
        content_type,
        metadata_json,
        is_archived,
    )
    .await?;

    // Log activity
    if let Err(e) = log_update(db, "bookmark".to_string(), bookmark.id.clone()).await {
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
pub async fn delete_bookmark(state: State<'_, DbState>, id: String) -> Result<()> {
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }
    let db = connection::get_database(&*state);
    let repo = BookmarkRepository::new(db.clone());
    repo.delete(&id).await?;

    // Log activity
    if let Err(e) = log_delete(db, "bookmark".to_string(), id.clone()).await {
        tracing::warn!("Failed to log bookmark deletion activity for bookmark {}: {}", id, e);
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
        (status = 200, description = "Tags added to bookmark", body = crate::db::models::Bookmark),
        (status = 404, description = "Bookmark or tag not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn add_tags_to_bookmark(
    state: State<'_, DbState>,
    id: String,
    tag_ids: Vec<String>,
) -> Result<crate::db::models::Bookmark> {
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }
    let db = connection::get_database(&*state);
    let repo = BookmarkRepository::new(db.clone());
    repo.add_tags(&id, tag_ids).await?;

    // Log activity
    if let Err(e) = log_tag_operation(db.clone(), "add_tags", "bookmark".to_string(), id.clone()).await {
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
        (status = 200, description = "Tags removed from bookmark", body = crate::db::models::Bookmark),
        (status = 404, description = "Bookmark or tag not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn remove_tags_from_bookmark(
    state: State<'_, DbState>,
    id: String,
    tag_ids: Vec<String>,
) -> Result<crate::db::models::Bookmark> {
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }
    let db = connection::get_database(&*state);
    let repo = BookmarkRepository::new(db.clone());
    repo.remove_tags(&id, tag_ids).await?;

    // Log activity
    if let Err(e) = log_tag_operation(db.clone(), "remove_tags", "bookmark".to_string(), id.clone()).await {
        tracing::warn!("Failed to log remove_tags activity for bookmark {}: {}", id, e);
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
        (status = 200, description = "Extracted metadata", body = crate::utils::metadata::extractor::ExtractedMetadata),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn extract_metadata(url: String) -> Result<crate::utils::metadata::extractor::ExtractedMetadata> {
    if url.is_empty() {
        return Err(AppError::BadRequest("URL is required".to_string()));
    }
    MetadataExtractor::extract(&url).await
}
