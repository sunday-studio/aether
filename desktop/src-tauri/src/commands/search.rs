use crate::commands::params::{EmptyPathParams, EmptyRequest, SearchQueryParams};
use crate::db::{connection, DbState};
use crate::db::repositories::{SearchDocumentQuery, SearchDocumentRepository};
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
    pub query: String,
    pub mode: String,
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
        ("offset" = Option<u32>, Query, description = "Pagination offset"),
        ("cursor" = Option<String>, Query, description = "Reserved cursor pagination token"),
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
    let params = query_params.ok_or_else(|| AppError::BadRequest("Query parameters are required".to_string()))?;
    if params.q.trim().is_empty() {
        return Err(AppError::BadRequest("Query parameter 'q' is required and cannot be empty".to_string()));
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

    if params.tags.as_ref().is_some_and(|tags| !tags.trim().is_empty()) {
        tracing::debug!("Search tag filtering is reserved for a later phase");
    }

    let requested_mode = params.mode.as_deref().unwrap_or("keyword").to_string();
    let mode = match requested_mode.as_str() {
        "keyword" | "fuzzy" => "keyword",
        "semantic" | "similar" => {
            return Err(AppError::BadRequest(
                "Semantic search is not available until embeddings are indexed".to_string(),
            ));
        }
        "hybrid" => {
            return Err(AppError::BadRequest(
                "Hybrid search is not available until embeddings are indexed".to_string(),
            ));
        }
        _ => "keyword",
    };

    let repo = SearchDocumentRepository::new(connection::get_database(&*state));
    let results = repo
        .search_keyword(
            &params.q,
            SearchDocumentQuery {
                resource_types,
                date_from: params.date_from,
                date_to: params.date_to,
                limit: params.limit,
                offset: params.offset,
            },
        )
        .await?;
    let total = results.len();
    let response = SearchResponse {
        results: results
            .into_iter()
            .map(|result| SearchResultResponse {
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
            })
            .collect(),
        total,
        query: params.q,
        mode: mode.to_string(),
    };

    Ok(response)
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
    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
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
