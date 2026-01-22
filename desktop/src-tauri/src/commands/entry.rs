use crate::db::{connection, DbState, EntryRepository};
use crate::db::repositories::LinkRepository;
use crate::error::{AppError, Result};
use crate::handlers::entry::CreateEntryRequest;
use crate::utils::{log_create, log_delete, log_tag_operation, log_update};
use crate::utils::link_parser::extract_links_from_lexical_content;
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
        (status = 200, description = "Created entry", body = crate::db::models::Entry),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command(rename_all = "camelCase")]
pub async fn create_entry(
    state: State<'_, DbState>,
    document: String,
    date: Option<chrono::DateTime<chrono::Utc>>,
    is_pinned: Option<bool>,
    is_archived: Option<bool>,
    is_deleted: Option<bool>,
) -> Result<crate::db::models::Entry> {
    let date = date.unwrap_or_else(chrono::Utc::now);
    let db = connection::get_database(&*state);
    let repo = EntryRepository::new(db.clone());
    let entry = repo.create(
        document.clone(),
        date,
        is_pinned.unwrap_or(false),
        is_archived.unwrap_or(false),
        is_deleted.unwrap_or(false),
    )
    .await?;
    
    // Sync links from content
    let link_repo = LinkRepository::new(db.clone());
    if let Ok(extracted_links) = extract_links_from_lexical_content(&document) {
        for link in extracted_links {
            let _ = link_repo.create(
                "entry".to_string(),
                entry.id.clone(),
                link.target_type,
                link.target_id,
                link.link_text,
            ).await;
        }
    }
    
    // Log activity
    if let Err(e) = log_create(db, "entry".to_string(), entry.id.clone()).await {
        tracing::warn!("Failed to log entry creation activity: {}", e);
    }
    
    Ok(entry)
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
        if let Err(e) = log_create(db.clone(), "entry".to_string(), entry.id.clone()).await {
            tracing::warn!("Failed to log entry creation activity for entry {}: {}", entry.id, e);
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
        (status = 200, description = "Updated entry", body = crate::db::models::Entry),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Entry not found"),
        (status = 409, description = "Conflict: Record was modified by another device"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command(rename_all = "camelCase")]
pub async fn update_entry(
    state: State<'_, DbState>,
    id: String,
    document: Option<String>,
    is_pinned: Option<bool>,
    is_archived: Option<bool>,
    is_deleted: Option<bool>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<crate::db::models::Entry> {
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }

    // If document is not provided, fetch the current entry to preserve it
    let document = if let Some(doc) = document {
        doc
    } else {
        let repo = EntryRepository::new(connection::get_database(&*state));
        let current_entry = repo.find_by_id(&id).await?
            .ok_or_else(|| AppError::NotFound(format!("Entry {} not found", id)))?;
        current_entry.document
    };

    let db = connection::get_database(&*state);
    let repo = EntryRepository::new(db.clone());
    let entry = repo.update(
        &id,
        document.clone(),
        is_pinned.unwrap_or(false),
        is_archived.unwrap_or(false),
        is_deleted.unwrap_or(false),
        updated_at,
    )
    .await?;
    
    // Sync links from content
    let link_repo = LinkRepository::new(db.clone());
    if let Ok(extracted_links) = extract_links_from_lexical_content(&document) {
        // Get existing links
        let existing_links = link_repo.find_by_source("entry", &id).await.unwrap_or_default();
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
            let target_key = (existing_link.target_type.clone(), existing_link.target_id.clone());
            if !new_targets.contains(&target_key) {
                let _ = link_repo.delete(
                    &existing_link.source_type,
                    &existing_link.source_id,
                    &existing_link.target_type,
                    &existing_link.target_id,
                ).await;
            }
        }
        
        // Create new links
        for link in extracted_links {
            let target_key = (link.target_type.clone(), link.target_id.clone());
            if !existing_targets.contains(&target_key) {
                let _ = link_repo.create(
                    "entry".to_string(),
                    id.clone(),
                    link.target_type,
                    link.target_id,
                    link.link_text,
                ).await;
            }
        }
    }
    
    // Log activity
    if let Err(e) = log_update(db, "entry".to_string(), entry.id.clone()).await {
        tracing::warn!("Failed to log entry update activity: {}", e);
    }
    
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
pub async fn delete_entry(state: State<'_, DbState>, id: String) -> Result<()> {
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }
    let db = connection::get_database(&*state);
    let repo = EntryRepository::new(db.clone());
    repo.delete(&id).await?;
    
    // Delete all links from this entry
    let link_repo = LinkRepository::new(db.clone());
    let _ = link_repo.delete_by_source("entry", &id).await;
    
    // Log activity
    if let Err(e) = log_delete(db, "entry".to_string(), id.clone()).await {
        tracing::warn!("Failed to log entry deletion activity for entry {}: {}", id, e);
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
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }
    let db = connection::get_database(&*state);
    let repo = EntryRepository::new(db.clone());
    repo.add_tags(&id, tag_ids).await?;
    
    // Log activity
    if let Err(e) = log_tag_operation(db.clone(), "add_tags", "entry".to_string(), id.clone()).await {
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
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }
    if tag_id.is_empty() {
        return Err(AppError::BadRequest("Tag ID is required".to_string()));
    }
    let db = connection::get_database(&*state);
    let repo = EntryRepository::new(db.clone());
    repo.remove_tags(&id, tag_id).await?;
    
    // Log activity
    if let Err(e) = log_tag_operation(db.clone(), "remove_tags", "entry".to_string(), id.clone()).await {
        tracing::warn!("Failed to log remove_tags activity for entry {}: {}", id, e);
    }
    
    // Return updated entry
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Entry {} not found", id)))
}
