use crate::commands::common::PaginationResponse;
use crate::commands::params::{
    EmptyPathParams, EmptyQueryParams, EmptyRequest, IdPathParams, PaginationQueryParams,
};
use crate::db::models::Entry;
use crate::db::repositories::LinkRepository;
use crate::db::{
    connection, DbState, EntryRepository, SearchDocumentRepository, SearchEmbeddingRepository,
};
use crate::error::{AppError, Result};
use crate::utils::link_parser::extract_links_from_lexical_content;
use crate::utils::performance_ledger::{record_rust_timing, PerfTimer};
use crate::utils::{log_create, log_delete, log_tag_operation, log_update};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use std::time::Instant;
use tauri::State;
use utoipa::ToSchema;

fn default_created_at() -> chrono::DateTime<Utc> {
    Utc::now()
}

async fn reindex_entry_search(db: std::sync::Arc<libsql::Database>, entry_id: &str) {
    let timer = PerfTimer::start("rust-phase", "entry.search_refresh");
    let document_started = Instant::now();
    if let Err(e) = SearchDocumentRepository::new(db.clone())
        .reindex_resource("entry", entry_id)
        .await
    {
        tracing::warn!("Failed to reindex entry {} for search: {}", entry_id, e);
        timer.finish(json!({
            "resource_type": "entry",
            "resource_id": entry_id,
            "status": "search_document_error",
            "error": e.to_string(),
        }));
        return;
    }
    let search_document_ms = document_started.elapsed().as_secs_f64() * 1000.0;

    let embeddings_started = Instant::now();
    if let Err(e) = SearchEmbeddingRepository::new(db)
        .refresh_existing_resource_embeddings("entry", entry_id)
        .await
    {
        tracing::warn!(
            "Failed to refresh entry {} search embeddings: {}",
            entry_id,
            e
        );
        timer.finish(json!({
            "resource_type": "entry",
            "resource_id": entry_id,
            "status": "search_embedding_error",
            "search_document_ms": (search_document_ms * 10.0).round() / 10.0,
            "error": e.to_string(),
        }));
        return;
    }
    let search_embedding_ms = embeddings_started.elapsed().as_secs_f64() * 1000.0;
    timer.finish(json!({
        "resource_type": "entry",
        "resource_id": entry_id,
        "status": "ok",
        "search_document_ms": (search_document_ms * 10.0).round() / 10.0,
        "search_embedding_ms": (search_embedding_ms * 10.0).round() / 10.0,
    }));
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
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

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
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

/// Request to add tags to an entry
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct AddTagsToEntryRequest {
    pub tag_ids: Vec<String>,
}

/// Request to remove tags from an entry
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RemoveTagFromEntryRequest {
    pub tag_ids: Vec<String>,
}

/// Get all entries
#[utoipa::path(
    get,
    path = "/v1/entry",
    tag = "Entries",
    params(
        ("limit" = Option<u32>, Query, description = "Number of entries per page (max 1000)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination")
    ),
    responses(
        (status = 200, description = "Paginated list of entries", body = PaginatedEntries),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_entries(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<PaginationQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<PaginationResponse<Entry>> {
    let _guard = connection::with_db_access(&*state).await;
    let params = query_params.unwrap_or_default();
    let repo = EntryRepository::new(connection::get_database(&*state));
    let (entries, next_cursor, has_more) = repo
        .find_all(params.normalize_limit(), params.cursor)
        .await?;
    Ok(PaginationResponse::new(entries, next_cursor, has_more))
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
        (status = 200, description = "Entry found", body = Entry),
        (status = 404, description = "Entry not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_entry_by_id(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<Entry> {
    let _guard = connection::with_db_access(&*state).await;
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }
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
        (status = 200, description = "Created entry", body = Entry),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn create_entry(
    state: State<'_, DbState>,
    request_data: Option<CreateEntryRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<Entry> {
    let command_started = Instant::now();
    if let Some(ref req) = request_data {
        tracing::info!(
            "create_entry called with request_data: document len = {}, date = {}, is_pinned = {:?}, is_archived = {:?}, is_deleted = {:?}",
            req.document.len(),
            req.date,
            req.is_pinned,
            req.is_archived,
            req.is_deleted
        );
    } else {
        tracing::info!("create_entry called with no request_data");
    }
    let db_gate_started = Instant::now();
    let _guard = connection::with_db_access(&*state).await;
    let db_gate_ms = db_gate_started.elapsed().as_secs_f64() * 1000.0;
    let request =
        request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    let db = connection::get_database(&*state);
    let repo = EntryRepository::new(db.clone());
    let repo_started = Instant::now();
    let entry = repo
        .create(
            request.document.clone(),
            request.date,
            request.is_pinned.unwrap_or(false),
            request.is_archived.unwrap_or(false),
            request.is_deleted.unwrap_or(false),
        )
        .await?;
    let repo_ms = repo_started.elapsed().as_secs_f64() * 1000.0;

    // Sync links from content
    let links_started = Instant::now();
    let link_repo = LinkRepository::new(db.clone());
    if let Ok(extracted_links) = extract_links_from_lexical_content(&request.document) {
        for link in extracted_links {
            let _ = link_repo
                .create(
                    "entry".to_string(),
                    entry.id.clone(),
                    link.target_type,
                    link.target_id,
                    link.link_text,
                )
                .await;
        }
    }
    let links_ms = links_started.elapsed().as_secs_f64() * 1000.0;

    let search_started = Instant::now();
    reindex_entry_search(db.clone(), &entry.id).await;
    let search_ms = search_started.elapsed().as_secs_f64() * 1000.0;

    // Log activity
    let activity_started = Instant::now();
    if let Err(e) = log_create(db.clone(), "entry".to_string(), entry.id.clone()).await {
        tracing::warn!("Failed to log entry creation activity: {}", e);
    }
    let activity_ms = activity_started.elapsed().as_secs_f64() * 1000.0;

    record_rust_timing(
        "rust-command",
        "create_entry",
        command_started.elapsed(),
        json!({
            "resource_type": "entry",
            "resource_id": entry.id,
            "document_bytes": request.document.len(),
            "db_gate_ms": (db_gate_ms * 10.0).round() / 10.0,
            "repo_create_ms": (repo_ms * 10.0).round() / 10.0,
            "links_ms": (links_ms * 10.0).round() / 10.0,
            "search_refresh_ms": (search_ms * 10.0).round() / 10.0,
            "activity_log_ms": (activity_ms * 10.0).round() / 10.0,
        }),
    );

    Ok(entry)
}

/// Bulk create entries
#[utoipa::path(
    post,
    path = "/v1/entry/bulk-create",
    tag = "Entries",
    request_body = Vec<CreateEntryRequest>,
    responses(
        (status = 200, description = "Created entries", body = Vec<Entry>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn bulk_create_entries(
    state: State<'_, DbState>,
    request_data: Option<Vec<CreateEntryRequest>>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<Vec<Entry>> {
    let _guard = connection::with_db_access(&*state).await;
    let payload =
        request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    let db = connection::get_database(&*state);
    let repo = EntryRepository::new(db.clone());
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

    // Log activities for each created entry
    for entry in &entries {
        reindex_entry_search(db.clone(), &entry.id).await;

        if let Err(e) = log_create(db.clone(), "entry".to_string(), entry.id.clone()).await {
            tracing::warn!(
                "Failed to log entry creation activity for entry {}: {}",
                entry.id,
                e
            );
        }
    }

    Ok(entries)
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
        (status = 200, description = "Updated entry", body = Entry),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Entry not found"),
        (status = 409, description = "Conflict: Record was modified by another device"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn update_entry(
    state: State<'_, DbState>,
    request_data: Option<UpdateEntryRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<Entry> {
    let command_started = Instant::now();
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }

    let db_gate_started = Instant::now();
    let _guard = connection::with_db_access(&*state).await;
    let db_gate_ms = db_gate_started.elapsed().as_secs_f64() * 1000.0;
    let request =
        request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;

    let db = connection::get_database(&*state);
    let repo = EntryRepository::new(db.clone());
    let repo_started = Instant::now();
    let entry = repo
        .update(
            &id,
            request.document.clone(),
            request.is_pinned.unwrap_or(false),
            request.is_archived.unwrap_or(false),
            request.is_deleted.unwrap_or(false),
            request.updated_at,
        )
        .await?;
    let repo_ms = repo_started.elapsed().as_secs_f64() * 1000.0;

    // Sync links from content
    let links_started = Instant::now();
    let link_repo = LinkRepository::new(db.clone());
    if let Ok(extracted_links) = extract_links_from_lexical_content(&request.document) {
        // Get existing links
        let existing_links = link_repo
            .find_by_source("entry", &id)
            .await
            .unwrap_or_default();
        let existing_targets: std::collections::HashSet<(String, String)> = existing_links
            .iter()
            .map(|l| (l.target_type.clone(), l.target_id.clone()))
            .collect();

        // Create new links
        let new_targets: std::collections::HashSet<(String, String)> = extracted_links
            .iter()
            .map(|l| (l.target_type.clone(), l.target_id.clone()))
            .collect();

        // Delete removed links
        for existing_link in &existing_links {
            let target_key = (
                existing_link.target_type.clone(),
                existing_link.target_id.clone(),
            );
            if !new_targets.contains(&target_key) {
                let _ = link_repo
                    .delete(
                        &existing_link.source_type,
                        &existing_link.source_id,
                        &existing_link.target_type,
                        &existing_link.target_id,
                    )
                    .await;
            }
        }

        // Create new links
        for link in extracted_links {
            let target_key = (link.target_type.clone(), link.target_id.clone());
            if !existing_targets.contains(&target_key) {
                let _ = link_repo
                    .create(
                        "entry".to_string(),
                        id.clone(),
                        link.target_type,
                        link.target_id,
                        link.link_text,
                    )
                    .await;
            }
        }
    }
    let links_ms = links_started.elapsed().as_secs_f64() * 1000.0;

    let search_started = Instant::now();
    reindex_entry_search(db.clone(), &entry.id).await;
    let search_ms = search_started.elapsed().as_secs_f64() * 1000.0;

    // Log activity
    let activity_started = Instant::now();
    if let Err(e) = log_update(db.clone(), "entry".to_string(), entry.id.clone()).await {
        tracing::warn!("Failed to log entry update activity: {}", e);
    }
    let activity_ms = activity_started.elapsed().as_secs_f64() * 1000.0;

    record_rust_timing(
        "rust-command",
        "update_entry",
        command_started.elapsed(),
        json!({
            "resource_type": "entry",
            "resource_id": entry.id,
            "document_bytes": request.document.len(),
            "db_gate_ms": (db_gate_ms * 10.0).round() / 10.0,
            "repo_update_ms": (repo_ms * 10.0).round() / 10.0,
            "links_ms": (links_ms * 10.0).round() / 10.0,
            "search_refresh_ms": (search_ms * 10.0).round() / 10.0,
            "activity_log_ms": (activity_ms * 10.0).round() / 10.0,
        }),
    );

    Ok(entry)
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
pub async fn delete_entry(
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
    let repo = EntryRepository::new(db.clone());
    repo.delete(&id).await?;

    // Delete all links from this entry
    let link_repo = LinkRepository::new(db.clone());
    let _ = link_repo.delete_by_source("entry", &id).await;

    reindex_entry_search(db.clone(), &id).await;

    // Log activity
    if let Err(e) = log_delete(db.clone(), "entry".to_string(), id.clone()).await {
        tracing::warn!(
            "Failed to log entry deletion activity for entry {}: {}",
            id,
            e
        );
    }

    Ok(())
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
        (status = 200, description = "Tags added to entry", body = Entry),
        (status = 404, description = "Entry or tag not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn add_tags_to_entry(
    state: State<'_, DbState>,
    request_data: Option<AddTagsToEntryRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<Entry> {
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
    let repo = EntryRepository::new(db.clone());
    repo.add_tags(&id, request.tag_ids).await?;

    // Log activity
    if let Err(e) = log_tag_operation(db.clone(), "add_tags", "entry".to_string(), id.clone()).await
    {
        tracing::warn!("Failed to log add_tags activity for entry {}: {}", id, e);
    }

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
    request_body = Vec<String>,
    responses(
        (status = 200, description = "Tags removed from entry", body = Entry),
        (status = 404, description = "Entry or tag not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn remove_tags_from_entry(
    state: State<'_, DbState>,
    request_data: Option<RemoveTagFromEntryRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<Entry> {
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }
    let request =
        request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;

    if request.tag_ids.is_empty() {
        return Err(AppError::BadRequest("Tag IDs are required".to_string()));
    }

    let _guard = connection::with_db_access(&*state).await;
    let db = connection::get_database(&*state);
    let repo = EntryRepository::new(db.clone());
    repo.remove_tags(&id, request.tag_ids).await?;

    // Log activity
    if let Err(e) =
        log_tag_operation(db.clone(), "remove_tags", "entry".to_string(), id.clone()).await
    {
        tracing::warn!("Failed to log remove_tags activity for entry {}: {}", id, e);
    }

    // Return updated entry
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Entry {} not found", id)))
}
