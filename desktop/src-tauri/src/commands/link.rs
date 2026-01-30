use crate::commands::params::{
    EmptyPathParams, EmptyQueryParams, EmptyRequest, LinkQueryParams, PaginationQueryParams,
};
use crate::db::{connection, DbState};
use crate::db::models::ResourceLink;
use crate::db::repositories::{LinkRepository};
use crate::db::repositories::search::{SearchRepository, ResourceType};
use crate::error::{AppError, Result};
use crate::handlers::common::{PaginatedLinks, PaginationResponse};
use crate::utils::link_parser::extract_links_from_lexical_content;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use utoipa::ToSchema;

/// Request to create a link
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateLinkRequest {
    pub source_type: String,
    pub source_id: String,
    pub target_type: String,
    pub target_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_text: Option<String>,
}

/// Response for linkable resource search
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LinkableResource {
    pub id: String,
    pub resource_type: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
}

/// Response for backlinks
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BacklinkResponse {
    pub link: ResourceLink,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_title: Option<String>,
}

/// Create a link between resources
#[utoipa::path(
    post,
    path = "/v1/links",
    tag = "Links",
    request_body = CreateLinkRequest,
    responses(
        (status = 200, description = "Created link", body = ResourceLink),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn create_link(
    state: State<'_, DbState>,
    request_data: Option<CreateLinkRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<ResourceLink> {
    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    // Validate resource types
    let valid_types = ["entry", "task", "goal", "canvas", "bookmark"];
    if !valid_types.contains(&request.source_type.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid source_type: {}. Must be one of: {:?}",
            request.source_type, valid_types
        )));
    }
    if !valid_types.contains(&request.target_type.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid target_type: {}. Must be one of: {:?}",
            request.target_type, valid_types
        )));
    }

    let db = connection::get_database(&*state);
    let repo = LinkRepository::new(db.clone());
    
    repo.create(request.source_type, request.source_id, request.target_type, request.target_id, request.link_text)
        .await
}

/// Get all backlinks to a target resource
#[utoipa::path(
    get,
    path = "/v1/links/backlinks",
    tag = "Links",
    params(
        ("targetType" = String, Query, description = "Target resource type"),
        ("targetId" = String, Query, description = "Target resource ID")
    ),
    responses(
        (status = 200, description = "List of backlinks", body = Vec<BacklinkResponse>),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_backlinks(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<LinkQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<Vec<BacklinkResponse>> {
    let params = query_params.ok_or_else(|| AppError::BadRequest("Query parameters are required".to_string()))?;
    let db = connection::get_database(&*state);
    let repo = LinkRepository::new(db.clone());
    
    let links = repo.find_by_target(&params.resource_type, &params.resource_id).await?;
    
    // Enrich with source titles
    let mut backlinks = Vec::new();
    for link in links {
        let source_title = get_resource_title(db.clone(), &link.source_type, &link.source_id).await?;
        backlinks.push(BacklinkResponse {
            link,
            source_title,
        });
    }
    
    Ok(backlinks)
}

/// Get all outgoing links from a source resource
#[utoipa::path(
    get,
    path = "/v1/links/outgoing",
    tag = "Links",
    params(
        ("sourceType" = String, Query, description = "Source resource type"),
        ("sourceId" = String, Query, description = "Source resource ID")
    ),
    responses(
        (status = 200, description = "List of outgoing links", body = Vec<ResourceLink>),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_outgoing_links(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<LinkQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<Vec<ResourceLink>> {
    let params = query_params.ok_or_else(|| AppError::BadRequest("Query parameters are required".to_string()))?;
    let db = connection::get_database(&*state);
    let repo = LinkRepository::new(db.clone());
    
    repo.find_by_source(&params.resource_type, &params.resource_id).await
}

/// Delete a link
#[utoipa::path(
    delete,
    path = "/v1/links",
    tag = "Links",
    params(
        ("sourceType" = String, Query, description = "Source resource type"),
        ("sourceId" = String, Query, description = "Source resource ID"),
        ("targetType" = String, Query, description = "Target resource type"),
        ("targetId" = String, Query, description = "Target resource ID")
    ),
    responses(
        (status = 200, description = "Link deleted"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn delete_link(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<DeleteLinkQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<()> {
    let params = query_params.ok_or_else(|| AppError::BadRequest("Query parameters are required".to_string()))?;
    let db = connection::get_database(&*state);
    let repo = LinkRepository::new(db.clone());
    
    repo.delete(&params.source_type, &params.source_id, &params.target_type, &params.target_id)
        .await
}

/// Search for resources to link (for autocomplete)
#[utoipa::path(
    get,
    path = "/v1/links/search",
    tag = "Links",
    params(
        ("q" = String, Query, description = "Search query"),
        ("types" = Option<String>, Query, description = "Comma-separated resource types to filter"),
        ("limit" = Option<u32>, Query, description = "Maximum number of results (default: 20)")
    ),
    responses(
        (status = 200, description = "List of linkable resources", body = Vec<LinkableResource>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn search_linkable_resources(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<SearchLinkableQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<Vec<LinkableResource>> {
    let params = query_params.ok_or_else(|| AppError::BadRequest("Query parameters are required".to_string()))?;
    if params.q.trim().is_empty() {
        return Err(AppError::BadRequest("Query parameter 'q' is required".to_string()));
    }

    // Parse resource types
    let resource_types = if let Some(ref types_str) = params.types {
        let type_vec: Vec<ResourceType> = types_str
            .split(',')
            .filter_map(|s| ResourceType::from_str(s.trim()))
            .collect();
        if type_vec.is_empty() {
            None
        } else {
            Some(type_vec)
        }
    } else {
        None
    };

    let limit = params.limit.unwrap_or(20).min(100);
    let db = connection::get_database(&*state);
    let search_repo = SearchRepository::new(db.clone());
    
    let results: Vec<crate::db::repositories::search::SearchResult> = search_repo
        .search_fuzzy(&params.q, resource_types, None, Some(limit), Some(0))
        .await?;

    let linkable_resources: Vec<LinkableResource> = results
        .into_iter()
        .filter_map(|result| {
            match result {
                crate::db::repositories::search::SearchResult::Entry { entry, .. } => {
                    Some(LinkableResource {
                        id: entry.id.clone(),
                        resource_type: "entry".to_string(),
                        title: extract_first_line_from_lexical(&entry.document).unwrap_or_default(),
                        preview: None,
                    })
                }
                crate::db::repositories::search::SearchResult::Task { task, .. } => {
                    Some(LinkableResource {
                        id: task.id.clone(),
                        resource_type: "task".to_string(),
                        title: task.title.clone(),
                        preview: task.description.clone(),
                    })
                }
                crate::db::repositories::search::SearchResult::SubTask { subtask, .. } => {
                    Some(LinkableResource {
                        id: subtask.id.clone(),
                        resource_type: "subtask".to_string(),
                        title: subtask.title.clone(),
                        preview: None,
                    })
                }
                crate::db::repositories::search::SearchResult::Goal { goal, .. } => {
                    Some(LinkableResource {
                        id: goal.id.clone(),
                        resource_type: "goal".to_string(),
                        title: goal.name.clone(),
                        preview: goal.description.clone(),
                    })
                }
                crate::db::repositories::search::SearchResult::Tag { tag, .. } => {
                    Some(LinkableResource {
                        id: tag.id.clone(),
                        resource_type: "tag".to_string(),
                        title: tag.name.clone(),
                        preview: None,
                    })
                }
                crate::db::repositories::search::SearchResult::Bookmark { bookmark, .. } => {
                    Some(LinkableResource {
                        id: bookmark.id.clone(),
                        resource_type: "bookmark".to_string(),
                        title: bookmark.title.clone().unwrap_or_default(),
                        preview: bookmark.description.clone(),
                    })
                }
            }
        })
        .collect();

    Ok(linkable_resources)
}

/// Get all links for graph visualization
#[utoipa::path(
    get,
    path = "/v1/links/graph",
    tag = "Links",
    params(
        ("limit" = Option<u32>, Query, description = "Number of links per page (max 1000)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination")
    ),
    responses(
        (status = 200, description = "Paginated list of links for graph", body = PaginatedLinks),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_all_links_for_graph(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<PaginationQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<PaginationResponse<ResourceLink>> {
    let params = query_params.unwrap_or_default();
    let db = connection::get_database(&*state);
    let repo = LinkRepository::new(db.clone());
    
    let (links, next_cursor, has_more) = repo
        .find_all_for_graph(params.normalize_limit(), params.cursor)
        .await?;
    Ok(PaginationResponse::new(links, next_cursor, has_more))
}

/// Sync links from content (extract and create/update links)
#[utoipa::path(
    post,
    path = "/v1/links/sync",
    tag = "Links",
    request_body = SyncLinksRequest,
    responses(
        (status = 200, description = "Links synced"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn sync_links_from_content(
    state: State<'_, DbState>,
    request_data: Option<SyncLinksRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<serde_json::Value> {
    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    let db = connection::get_database(&*state);
    let link_repo = LinkRepository::new(db.clone());
    
    // Extract links from content
    let extracted_links = extract_links_from_lexical_content(&request.content)?;
    
    // Get existing links
    let existing_links = link_repo.find_by_source(&request.source_type, &request.source_id).await?;
    
    // Create a set of existing target IDs for quick lookup
    let existing_targets: std::collections::HashSet<(String, String)> = existing_links
        .iter()
        .map(|l| (l.target_type.clone(), l.target_id.clone()))
        .collect();
    
    // Create a set of new target IDs
    let new_targets: std::collections::HashSet<(String, String)> = extracted_links
        .iter()
        .map(|l| (l.target_type.clone(), l.target_id.clone()))
        .collect();
    
    // Delete links that are no longer in content
    for existing_link in &existing_links {
        let target_key = (existing_link.target_type.clone(), existing_link.target_id.clone());
        if !new_targets.contains(&target_key) {
            link_repo
                .delete(
                    &existing_link.source_type,
                    &existing_link.source_id,
                    &existing_link.target_type,
                    &existing_link.target_id,
                )
                .await?;
        }
    }
    
    // Create new links
    for extracted_link in extracted_links {
        let target_key = (extracted_link.target_type.clone(), extracted_link.target_id.clone());
        if !existing_targets.contains(&target_key) {
            link_repo
                .create(
                    request.source_type.clone(),
                    request.source_id.clone(),
                    extracted_link.target_type,
                    extracted_link.target_id,
                    extracted_link.link_text,
                )
                .await?;
        }
    }
    
    Ok(serde_json::json!({"success": true}))
}

/// Helper function to get resource title
async fn get_resource_title(
    db: Arc<libsql::Database>,
    resource_type: &str,
    resource_id: &str,
) -> Result<Option<String>> {
    use crate::db::repositories::{
        BookmarkRepository, CanvasRepository, EntryRepository, GoalRepository, TaskRepository,
    };
    
    match resource_type {
        "entry" => {
            let repo = EntryRepository::new(db);
            let entry = repo.find_by_id(resource_id).await?;
            Ok(entry.and_then(|e| {
                // Extract first line from document (Lexical JSON)
                extract_first_line_from_lexical(&e.document)
            }))
        }
        "task" => {
            let repo = TaskRepository::new(db);
            let task = repo.find_by_id(resource_id).await?;
            Ok(task.map(|t| t.title))
        }
        "goal" => {
            let repo = GoalRepository::new(db);
            let goal = repo.find_by_id(resource_id).await?;
            Ok(goal.map(|g| g.name))
        }
        "canvas" => {
            let repo = CanvasRepository::new(db);
            let canvas = repo.find_by_id(resource_id).await?;
            Ok(canvas.map(|c| c.name))
        }
        "bookmark" => {
            let repo = BookmarkRepository::new(db);
            let bookmark = repo.find_by_id(resource_id).await?;
            Ok(bookmark.and_then(|b| b.title))
        }
        _ => Ok(None),
    }
}

/// Extract first line from Lexical JSON content
fn extract_first_line_from_lexical(content: &str) -> Option<String> {
    use crate::utils::link_parser::extract_text_from_lexical_content;
    
    if let Ok(text) = extract_text_from_lexical_content(content) {
        text.lines().next().map(|s| s.trim().to_string())
    } else {
        None
    }
}

/// Request to sync links from content
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SyncLinksRequest {
    pub source_type: String,
    pub source_id: String,
    pub content: String,
}

/// Query parameters for linkable resource search
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchLinkableQueryParams {
    pub q: String,
    #[serde(default)]
    pub types: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
}

/// Query parameters for deleting a link
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteLinkQueryParams {
    pub source_type: String,
    pub source_id: String,
    pub target_type: String,
    pub target_id: String,
}
