use crate::commands::params::{EmptyPathParams, EmptyQueryParams, EmptyRequest, ModelNamePathParams};
use crate::error::Result;
use crate::utils::embeddings::model_manager;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size: String,
    pub dimensions: Option<u32>,
    pub file_size: u64,
    pub download_url: Option<String>,
    pub is_downloaded: bool,
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
        let is_downloaded = model_manager::is_model_downloaded(&model.name)
            .unwrap_or(false);
        
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
    ).await?;
    
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
