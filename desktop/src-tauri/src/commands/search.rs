use crate::commands::params::{EmptyPathParams, EmptyRequest, SearchQueryParams};
use crate::db::{connection, DbState};
use crate::db::repositories::search::{SearchRepository, ResourceType};
use crate::error::{AppError, Result};
use crate::handlers::search::{SearchResponse, SearchResultResponse};
use tauri::State;

/// Search across all resources
#[utoipa::path(
    get,
    path = "/v1/search",
    tag = "Search",
    params(
        ("q" = String, Query, description = "Search query"),
        ("types" = Option<String>, Query, description = "Comma-separated resource types: entry,task,subtask,goal,tag"),
        ("tags" = Option<String>, Query, description = "Comma-separated tag IDs to filter by"),
        ("limit" = Option<u32>, Query, description = "Maximum number of results (default: 50, max: 100)"),
        ("offset" = Option<u32>, Query, description = "Pagination offset"),
        ("mode" = Option<String>, Query, description = "Search mode: fuzzy, similar, or hybrid (default: fuzzy)")
    ),
    responses(
        (status = 200, description = "Search results", body = SearchResponse),
        (status = 400, description = "Bad request (empty query)"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn search_resources(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<SearchQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<SearchResponse> {
    let _guard = connection::with_db_access(&*state).await;
    let params = query_params.ok_or_else(|| AppError::BadRequest("Query parameters are required".to_string()))?;
    if params.q.trim().is_empty() {
        return Err(AppError::BadRequest("Query parameter 'q' is required and cannot be empty".to_string()));
    }

    // Parse resource types
    let types = if let Some(ref types_str) = params.types {
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

    // Parse tag IDs
    let tag_ids = if let Some(ref tags_str) = params.tags {
        let tag_vec: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if tag_vec.is_empty() {
            None
        } else {
            Some(tag_vec)
        }
    } else {
        None
    };

    let mode = params.mode.as_deref().unwrap_or("fuzzy").to_string();

    let repo = SearchRepository::new(connection::get_database(&*state));
    let results: Vec<crate::db::repositories::search::SearchResult> = repo.search_fuzzy(
        &params.q,
        types,
        tag_ids,
        params.limit,
        params.offset,
    ).await?;

    let total = results.len();
    let response = SearchResponse {
        results: results.into_iter().map(SearchResultResponse::from).collect(),
        total,
        query: params.q,
        mode,
    };

    Ok(response)
}
