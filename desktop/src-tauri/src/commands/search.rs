use crate::commands::params::{EmptyPathParams, EmptyRequest, SearchQueryParams};
use crate::db::{connection, DbState};
use crate::db::repositories::search::{SearchRepository, ResourceType, SearchResult};
use crate::db::repositories::SearchDocumentRepository;
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
    pub limit: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default)]
    pub mode: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub results: Vec<SearchResultResponse>,
    pub total: usize,
    pub query: String,
    pub mode: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum SearchResultResponse {
    Entry {
        id: String,
        #[serde(flatten)]
        entry: crate::db::models::Entry,
        score: f64,
        highlights: Vec<String>,
    },
    Task {
        id: String,
        #[serde(flatten)]
        task: crate::db::models::Task,
        score: f64,
        highlights: Vec<String>,
    },
    SubTask {
        id: String,
        #[serde(flatten)]
        subtask: crate::db::models::SubTask,
        score: f64,
        highlights: Vec<String>,
    },
    Goal {
        id: String,
        #[serde(flatten)]
        goal: crate::db::models::Goal,
        score: f64,
        highlights: Vec<String>,
    },
    Tag {
        id: String,
        #[serde(flatten)]
        tag: crate::db::models::Tag,
        score: f64,
        highlights: Vec<String>,
    },
    Bookmark {
        id: String,
        #[serde(flatten)]
        bookmark: crate::db::models::Bookmark,
        score: f64,
        highlights: Vec<String>,
    },
}

impl From<SearchResult> for SearchResultResponse {
    fn from(result: SearchResult) -> Self {
        match result {
            SearchResult::Entry { entry, score, highlights } => SearchResultResponse::Entry {
                id: entry.id.clone(),
                entry,
                score,
                highlights,
            },
            SearchResult::Task { task, score, highlights } => SearchResultResponse::Task {
                id: task.id.clone(),
                task,
                score,
                highlights,
            },
            SearchResult::SubTask { subtask, score, highlights } => SearchResultResponse::SubTask {
                id: subtask.id.clone(),
                subtask,
                score,
                highlights,
            },
            SearchResult::Goal { goal, score, highlights } => SearchResultResponse::Goal {
                id: goal.id.clone(),
                goal,
                score,
                highlights,
            },
            SearchResult::Tag { tag, score, highlights } => SearchResultResponse::Tag {
                id: tag.id.clone(),
                tag,
                score,
                highlights,
            },
            SearchResult::Bookmark { bookmark, score, highlights } => {
                SearchResultResponse::Bookmark {
                    id: bookmark.id.clone(),
                    bookmark,
                    score,
                    highlights,
                }
            }
        }
    }
}

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
    let results: Vec<SearchResult> = match mode.as_str() {
        "hybrid" => {
            repo.search_hybrid(
                &params.q,
                types,
                tag_ids,
                params.limit,
                params.offset,
            )
            .await?
        }
        "similar" => {
            repo.search_fuzzy(
                &params.q,
                types,
                tag_ids,
                params.limit,
                params.offset,
            )
            .await?
        }
        _ => {
            repo.search_fuzzy(
                &params.q,
                types,
                tag_ids,
                params.limit,
                params.offset,
            )
            .await?
        }
    };

    let total = results.len();
    let response = SearchResponse {
        results: results.into_iter().map(SearchResultResponse::from).collect(),
        total,
        query: params.q,
        mode,
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
