use crate::commands::params::{
    EmptyPathParams, EmptyRequest, RelatedSearchQueryParams, SearchQueryParams,
    WeekContextQueryParams,
};
use crate::db::repositories::{SearchDocumentQuery, SearchDocumentRepository};
use crate::db::{connection, DbState};
use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use tauri::State;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    pub q: String,
    #[serde(default)]
    pub types: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
    #[serde(default)]
    pub date_from: Option<String>,
    #[serde(default)]
    pub date_to: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default)]
    pub mode: Option<String>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReindexResourceRequest {
    pub resource_type: String,
    pub resource_id: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub results: Vec<SearchResultResponse>,
    pub total: usize,
    pub next_cursor: Option<String>,
    pub has_more: bool,
    pub query: String,
    pub mode: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchContextResponse {
    pub results: Vec<SearchResultResponse>,
    pub total: usize,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultResponse {
    pub id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub title: String,
    pub preview: String,
    pub score: f64,
    pub match_kind: String,
    pub highlights: Vec<String>,
    pub source_updated_at: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Search across all resources
#[utoipa::path(
    get,
    path = "/v1/search",
    tag = "Search",
    params(
        ("q" = String, Query, description = "Search query"),
        ("types" = Option<String>, Query, description = "Comma-separated resource types: entry,task,goal,tag,bookmark"),
        ("tags" = Option<String>, Query, description = "Comma-separated tag IDs to filter by"),
        ("date_from" = Option<String>, Query, description = "Filter source_updated_at at or after this ISO 8601 value"),
        ("date_to" = Option<String>, Query, description = "Filter source_updated_at at or before this ISO 8601 value"),
        ("limit" = Option<u32>, Query, description = "Maximum number of results (default: 50, max: 100)"),
        ("offset" = Option<u32>, Query, description = "Legacy pagination offset"),
        ("cursor" = Option<String>, Query, description = "Opaque pagination cursor"),
        ("mode" = Option<String>, Query, description = "Search mode: keyword, semantic, or hybrid (default: keyword)")
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
    let params = query_params
        .ok_or_else(|| AppError::BadRequest("Query parameters are required".to_string()))?;
    if params.q.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Query parameter 'q' is required and cannot be empty".to_string(),
        ));
    }

    let resource_types = if let Some(ref types_str) = params.types {
        let type_vec: Vec<String> = types_str
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| matches!(s.as_str(), "entry" | "task" | "goal" | "tag" | "bookmark"))
            .collect();
        if type_vec.is_empty() {
            None
        } else {
            Some(type_vec)
        }
    } else {
        None
    };

    let tag_ids = parse_tag_ids(params.tags.as_deref());

    let requested_mode = params.mode.as_deref().unwrap_or("keyword").to_string();
    let mode = resolve_search_mode(&requested_mode)?;

    let repo = SearchDocumentRepository::new(connection::get_database(&*state));
    let query = SearchDocumentQuery {
        resource_types,
        tag_ids,
        date_from: params.date_from,
        date_to: params.date_to,
        limit: params.limit,
        offset: params.offset,
        cursor: params.cursor,
    };
    let results = match mode {
        "semantic" => repo.search_semantic(&params.q, query).await?,
        "hybrid" => repo.search_hybrid(&params.q, query).await?,
        _ => repo.search_keyword(&params.q, query).await?,
    };
    let total = results.results.len();
    let response = SearchResponse {
        results: results
            .results
            .into_iter()
            .map(search_result_response)
            .collect(),
        total,
        next_cursor: results.next_cursor,
        has_more: results.has_more,
        query: params.q,
        mode: mode.to_string(),
    };

    Ok(response)
}

/// Find resources related to an indexed resource
#[utoipa::path(
    get,
    path = "/v1/search/related",
    tag = "Search",
    params(
        ("resource_type" = String, Query, description = "Resource type: entry, task, goal, tag, or bookmark"),
        ("resource_id" = String, Query, description = "Resource ID"),
        ("limit" = Option<u32>, Query, description = "Maximum number of related resources")
    ),
    responses(
        (status = 200, description = "Related resources", body = SearchContextResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn find_related_resources(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<RelatedSearchQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<SearchContextResponse> {
    let _guard = connection::with_db_access(&*state).await;
    let params = query_params
        .ok_or_else(|| AppError::BadRequest("Query parameters are required".to_string()))?;
    if params.resource_type.trim().is_empty() || params.resource_id.trim().is_empty() {
        return Err(AppError::BadRequest(
            "resourceType and resourceId are required".to_string(),
        ));
    }

    let repo = SearchDocumentRepository::new(connection::get_database(&*state));
    let results = repo
        .find_related(&params.resource_type, &params.resource_id, params.limit)
        .await?;
    Ok(search_context_response(results))
}

/// Retrieve clean context for a query
#[utoipa::path(
    get,
    path = "/v1/search/context",
    tag = "Search",
    params(
        ("q" = String, Query, description = "Context query"),
        ("types" = Option<String>, Query, description = "Comma-separated resource types"),
        ("tags" = Option<String>, Query, description = "Comma-separated tag IDs to filter by"),
        ("date_from" = Option<String>, Query, description = "Filter source_updated_at at or after this ISO 8601 value"),
        ("date_to" = Option<String>, Query, description = "Filter source_updated_at at or before this ISO 8601 value"),
        ("limit" = Option<u32>, Query, description = "Maximum number of context resources")
    ),
    responses(
        (status = 200, description = "Retrieved context", body = SearchContextResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn retrieve_context(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<SearchQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<SearchContextResponse> {
    let _guard = connection::with_db_access(&*state).await;
    let params = query_params
        .ok_or_else(|| AppError::BadRequest("Query parameters are required".to_string()))?;
    if params.q.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Query parameter 'q' is required and cannot be empty".to_string(),
        ));
    }

    let repo = SearchDocumentRepository::new(connection::get_database(&*state));
    let page = repo
        .search_keyword(
            &params.q,
            SearchDocumentQuery {
                resource_types: parse_resource_types(params.types.as_deref()),
                tag_ids: parse_tag_ids(params.tags.as_deref()),
                date_from: params.date_from,
                date_to: params.date_to,
                limit: params.limit,
                offset: None,
                cursor: None,
            },
        )
        .await?;
    Ok(search_context_response(page.results))
}

/// Retrieve clean context for a date range
#[utoipa::path(
    get,
    path = "/v1/search/week-context",
    tag = "Search",
    params(
        ("start_date" = String, Query, description = "Inclusive start ISO 8601 value"),
        ("end_date" = String, Query, description = "Inclusive end ISO 8601 value"),
        ("limit" = Option<u32>, Query, description = "Maximum number of context resources")
    ),
    responses(
        (status = 200, description = "Retrieved week context", body = SearchContextResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn retrieve_week_context(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<WeekContextQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<SearchContextResponse> {
    let _guard = connection::with_db_access(&*state).await;
    let params = query_params
        .ok_or_else(|| AppError::BadRequest("Query parameters are required".to_string()))?;
    if params.start_date.trim().is_empty() || params.end_date.trim().is_empty() {
        return Err(AppError::BadRequest(
            "startDate and endDate are required".to_string(),
        ));
    }

    let repo = SearchDocumentRepository::new(connection::get_database(&*state));
    let results = repo
        .list_context_by_date_range(&params.start_date, &params.end_date, params.limit)
        .await?;
    Ok(search_context_response(results))
}

fn parse_tag_ids(tags: Option<&str>) -> Option<Vec<String>> {
    tags.map(|tags| {
        tags.split(',')
            .map(|tag| tag.trim().to_string())
            .filter(|tag| !tag.is_empty())
            .collect::<Vec<_>>()
    })
    .filter(|tags| !tags.is_empty())
}

fn parse_resource_types(types: Option<&str>) -> Option<Vec<String>> {
    let resource_types = types?
        .split(',')
        .map(|resource_type| resource_type.trim().to_lowercase())
        .filter(|resource_type| {
            matches!(
                resource_type.as_str(),
                "entry" | "task" | "goal" | "tag" | "bookmark"
            )
        })
        .collect::<Vec<_>>();
    if resource_types.is_empty() {
        None
    } else {
        Some(resource_types)
    }
}

fn search_context_response(
    results: Vec<crate::db::repositories::search_document::SearchDocumentResult>,
) -> SearchContextResponse {
    let total = results.len();
    SearchContextResponse {
        results: results.into_iter().map(search_result_response).collect(),
        total,
    }
}

fn search_result_response(
    result: crate::db::repositories::search_document::SearchDocumentResult,
) -> SearchResultResponse {
    SearchResultResponse {
        id: result.id,
        resource_type: result.resource_type,
        resource_id: result.resource_id,
        title: result.title,
        preview: result.preview,
        score: result.score,
        match_kind: result.match_kind,
        highlights: result.highlights,
        source_updated_at: result.source_updated_at,
        created_at: result.created_at,
        updated_at: result.updated_at,
    }
}

fn resolve_search_mode(requested_mode: &str) -> Result<&'static str> {
    match requested_mode {
        "keyword" | "fuzzy" => Ok("keyword"),
        "semantic" | "similar" => Ok("semantic"),
        "hybrid" => Ok("hybrid"),
        _ => Ok("keyword"),
    }
}

/// Rebuild the local search document index
#[utoipa::path(
    post,
    path = "/v1/search/index/reindex",
    tag = "Search",
    responses(
        (status = 200, description = "Search index rebuilt", body = crate::db::repositories::SearchIndexStatus),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn reindex_search(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<crate::commands::params::EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<crate::db::repositories::SearchIndexStatus> {
    let _guard = connection::with_db_access(&*state).await;
    let repo = SearchDocumentRepository::new(connection::get_database(&*state));
    repo.reindex_all().await
}

/// Rebuild one local search document resource
#[utoipa::path(
    post,
    path = "/v1/search/index/resource",
    tag = "Search",
    request_body = ReindexResourceRequest,
    responses(
        (status = 200, description = "Resource search index rebuilt"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn reindex_search_resource(
    state: State<'_, DbState>,
    request_data: Option<ReindexResourceRequest>,
    _query_params: Option<crate::commands::params::EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<()> {
    let _guard = connection::with_db_access(&*state).await;
    let request =
        request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    if request.resource_type.trim().is_empty() || request.resource_id.trim().is_empty() {
        return Err(AppError::BadRequest(
            "resourceType and resourceId are required".to_string(),
        ));
    }

    let repo = SearchDocumentRepository::new(connection::get_database(&*state));
    repo.reindex_resource(&request.resource_type, &request.resource_id)
        .await
}

/// Get local search document index status
#[utoipa::path(
    get,
    path = "/v1/search/index/status",
    tag = "Search",
    responses(
        (status = 200, description = "Search index status", body = crate::db::repositories::SearchIndexStatus),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_search_index_status(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<crate::commands::params::EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<crate::db::repositories::SearchIndexStatus> {
    let _guard = connection::with_db_access(&*state).await;
    let repo = SearchDocumentRepository::new(connection::get_database(&*state));
    repo.status().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_search_mode_is_available() {
        let mode = resolve_search_mode("semantic").expect("semantic should resolve");

        assert_eq!(mode, "semantic");
    }

    #[test]
    fn hybrid_search_mode_is_available() {
        let mode = resolve_search_mode("hybrid").expect("hybrid should resolve");

        assert_eq!(mode, "hybrid");
    }
}
