use crate::commands::params::{
    EmptyPathParams, EmptyQueryParams, EmptyRequest, ModelNamePathParams,
};
use crate::db::repositories::{SearchEmbeddingRepository, SearchEmbeddingStatus};
use crate::db::{connection, DbState};
use crate::error::Result;
use crate::utils::embeddings::model_manager;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

const DEFAULT_SEARCH_EMBEDDING_MODEL: &str = "local-hash-384";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size: String,
    pub dimensions: Option<u32>,
    pub file_size: u64,
    pub download_url: Option<String>,
    pub is_downloaded: bool,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct IndexSearchEmbeddingsRequest {
    #[serde(default)]
    pub model_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct IndexSearchResourceEmbeddingsRequest {
    pub resource_type: String,
    pub resource_id: String,
    #[serde(default)]
    pub model_name: Option<String>,
}

/// List available embedding models
#[tauri::command]
pub async fn list_embedding_models(
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<Vec<ModelInfo>> {
    let models = model_manager::list_available_models();
    let mut result = Vec::new();

    for model in models {
        let is_downloaded = model_manager::is_model_downloaded(&model.name).unwrap_or(false);

        result.push(ModelInfo {
            name: model.name,
            size: model.size,
            dimensions: model.dimensions,
            file_size: model.file_size,
            download_url: model.download_url,
            is_downloaded,
        });
    }

    Ok(result)
}

/// Download an embedding model
#[tauri::command]
pub async fn download_embedding_model(
    _app: AppHandle,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<ModelNamePathParams>,
) -> Result<String> {
    let model_name = path_params
        .and_then(|p| Some(p.model_name))
        .ok_or_else(|| crate::error::AppError::BadRequest("Model name is required".to_string()))?;
    // Download with progress events
    // TODO: Implement progress events for Tauri 2
    model_manager::download_model(
        &model_name,
        None, // Progress callback disabled for now
    )
    .await?;

    Ok(format!("Model {} downloaded successfully", model_name))
}

/// Verify embedding model integrity
#[tauri::command]
pub async fn verify_embedding_model(
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<ModelNamePathParams>,
) -> Result<bool> {
    let model_name = path_params
        .and_then(|p| Some(p.model_name))
        .ok_or_else(|| crate::error::AppError::BadRequest("Model name is required".to_string()))?;
    model_manager::verify_model(&model_name)
}

/// Delete a downloaded embedding model
#[tauri::command]
pub async fn delete_embedding_model(
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<ModelNamePathParams>,
) -> Result<()> {
    let model_name = path_params
        .and_then(|p| Some(p.model_name))
        .ok_or_else(|| crate::error::AppError::BadRequest("Model name is required".to_string()))?;
    model_manager::delete_model(&model_name)
}

/// Index embeddings for all local search documents
#[utoipa::path(
    post,
    path = "/v1/search/embeddings/index",
    tag = "Search",
    request_body = IndexSearchEmbeddingsRequest,
    responses(
        (status = 200, description = "Search embeddings indexed", body = SearchEmbeddingStatus),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn index_search_embeddings(
    state: State<'_, DbState>,
    request_data: Option<IndexSearchEmbeddingsRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<SearchEmbeddingStatus> {
    let _guard = connection::with_db_access(&*state).await;
    let model_name = request_data
        .and_then(|request| request.model_name)
        .filter(|model_name| !model_name.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_SEARCH_EMBEDDING_MODEL.to_string());
    let repo = SearchEmbeddingRepository::new(connection::get_database(&*state));
    repo.index_all_embeddings(&model_name).await
}

/// Index embeddings for one local search resource
#[utoipa::path(
    post,
    path = "/v1/search/embeddings/resource",
    tag = "Search",
    request_body = IndexSearchResourceEmbeddingsRequest,
    responses(
        (status = 200, description = "Search resource embeddings indexed", body = SearchEmbeddingStatus),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn index_search_resource_embeddings(
    state: State<'_, DbState>,
    request_data: Option<IndexSearchResourceEmbeddingsRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<SearchEmbeddingStatus> {
    let _guard = connection::with_db_access(&*state).await;
    let request = request_data.ok_or_else(|| {
        crate::error::AppError::BadRequest("Request data is required".to_string())
    })?;
    if request.resource_type.trim().is_empty() || request.resource_id.trim().is_empty() {
        return Err(crate::error::AppError::BadRequest(
            "resourceType and resourceId are required".to_string(),
        ));
    }

    let model_name = request
        .model_name
        .filter(|model_name| !model_name.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_SEARCH_EMBEDDING_MODEL.to_string());
    let repo = SearchEmbeddingRepository::new(connection::get_database(&*state));
    repo.index_resource_embeddings(&request.resource_type, &request.resource_id, &model_name)
        .await
}

/// Get local search embedding index status
#[utoipa::path(
    get,
    path = "/v1/search/embeddings/status",
    tag = "Search",
    responses(
        (status = 200, description = "Search embedding status", body = SearchEmbeddingStatus),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_search_embedding_status(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<SearchEmbeddingStatus> {
    let _guard = connection::with_db_access(&*state).await;
    let repo = SearchEmbeddingRepository::new(connection::get_database(&*state));
    repo.status().await
}
