use crate::commands::params::{
    EmptyPathParams, EmptyQueryParams, EmptyRequest, MediaIdPathParams, ModelSizePathParams,
    PaginationQueryParams, TranscriptionIdPathParams, TranscriptionStartQueryParams,
};
use crate::db::connection;
use crate::db::repositories::TranscriptionRepository;
use crate::error::{AppError, Result};
use crate::commands::common::PaginationResponse;
use crate::settings;
use crate::transcription::model_manager;
use crate::transcription::providers::{GroqProvider, LocalWhisperProvider, OpenAIProvider, SelfHostedProvider};
use crate::transcription::provider::TranscriptionProvider;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use utoipa::ToSchema;

/// Request to set active transcription
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SetActiveTranscriptionRequest {
    pub transcription_id: String,
    pub media_id: String,
}

/// Request to validate provider
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ValidateProviderRequest {
    pub provider_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProviderInfo {
    pub name: String,
    pub display_name: String,
    pub requires_api_key: bool,
    pub requires_download: bool,
    pub status: String, // "ready" | "not_configured" | "downloading" | "error"
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelInfo {
    pub name: String,
    pub size: String,
    pub file_size: u64,
    pub download_url: String,
    pub is_downloaded: bool,
}

/// Start transcription for an audio file
#[utoipa::path(
    post,
    path = "/v1/transcription",
    tag = "Transcription",
    params(
        ("mediaId" = String, Path, description = "Media ID"),
        ("providerName" = Option<String>, Query, description = "Provider name (optional)")
    ),
    responses(
        (status = 200, description = "Transcription ID", body = String),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn start_transcription(
    app: AppHandle,
    state: State<'_, crate::DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<TranscriptionStartQueryParams>,
    path_params: Option<MediaIdPathParams>,
) -> Result<String> {
    let media_id = path_params
        .and_then(|p| Some(p.media_id))
        .ok_or_else(|| AppError::BadRequest("Media ID is required".to_string()))?;
    if media_id.is_empty() {
        return Err(AppError::BadRequest("Media ID is required".to_string()));
    }

    let _guard = connection::with_db_access(&*state).await;
    let database = connection::get_database(&*state);
    
    // Get provider name (default or specified)
    let provider = if let Some(ref params) = query_params {
        if let Some(name) = &params.provider {
            name.clone()
        } else {
            // Get from settings
            settings::get_setting(database.clone(), "transcription.default_provider")
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| "openai".to_string())
        }
    } else {
        // Get from settings
        settings::get_setting(database.clone(), "transcription.default_provider")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "openai".to_string())
    };

    // Create transcription record
    let repo = TranscriptionRepository::new(database.clone());
    let transcription = repo.create(media_id.clone(), provider.clone(), None).await?;
    
    // Create queue and enqueue job
    let queue = crate::transcription::TranscriptionQueue::new(database.clone(), app.clone());
    queue.enqueue(crate::transcription::TranscriptionJob {
        media_id,
        provider_name: provider,
        transcription_id: transcription.id.clone(),
    }).await?;
    
    Ok(transcription.id)
}

/// Get all transcriptions for an audio file
#[utoipa::path(
    get,
    path = "/v1/transcription/{mediaId}",
    tag = "Transcription",
    params(
        ("mediaId" = String, Path, description = "Media ID"),
        ("limit" = Option<u32>, Query, description = "Number of transcriptions per page (max 1000)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination")
    ),
    responses(
        (status = 200, description = "Paginated list of transcriptions", body = PaginatedTranscriptions),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_transcriptions(
    state: State<'_, crate::DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<PaginationQueryParams>,
    path_params: Option<MediaIdPathParams>,
) -> Result<PaginationResponse<crate::db::models::AudioTranscription>> {
    let media_id = path_params
        .and_then(|p| Some(p.media_id))
        .ok_or_else(|| AppError::BadRequest("Media ID is required".to_string()))?;
    if media_id.is_empty() {
        return Err(AppError::BadRequest("Media ID is required".to_string()));
    }

    let _guard = connection::with_db_access(&*state).await;
    let params = query_params.unwrap_or_default();
    let database = connection::get_database(&*state);
    let repo = TranscriptionRepository::new(database);
    let (transcriptions, next_cursor, has_more) = repo
        .find_by_media_id(&media_id, params.normalize_limit(), params.cursor)
        .await?;
    Ok(PaginationResponse::new(transcriptions, next_cursor, has_more))
}

/// Get a specific transcription by ID
#[utoipa::path(
    get,
    path = "/v1/transcription/by-id/{transcriptionId}",
    tag = "Transcription",
    params(
        ("transcriptionId" = String, Path, description = "Transcription ID")
    ),
    responses(
        (status = 200, description = "Transcription", body = Option<AudioTranscription>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_transcription_by_id(
    state: State<'_, crate::DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<TranscriptionIdPathParams>,
) -> Result<Option<crate::db::models::AudioTranscription>> {
    let transcription_id = path_params
        .and_then(|p| Some(p.transcription_id))
        .ok_or_else(|| AppError::BadRequest("Transcription ID is required".to_string()))?;
    if transcription_id.is_empty() {
        return Err(AppError::BadRequest("Transcription ID is required".to_string()));
    }

    let _guard = connection::with_db_access(&*state).await;
    let database = connection::get_database(&*state);
    let repo = TranscriptionRepository::new(database);
    repo.find_by_id(&transcription_id).await
}

/// Set active transcription
#[utoipa::path(
    post,
    path = "/v1/transcription/set-active",
    tag = "Transcription",
    params(
        ("transcriptionId" = String, Path, description = "Transcription ID"),
        ("mediaId" = String, Path, description = "Media ID")
    ),
    responses(
        (status = 200, description = "Transcription set as active"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn set_active_transcription(
    state: State<'_, crate::DbState>,
    request_data: Option<SetActiveTranscriptionRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<()> {
    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    if request.transcription_id.is_empty() || request.media_id.is_empty() {
        return Err(AppError::BadRequest("Transcription ID and Media ID are required".to_string()));
    }

    let _guard = connection::with_db_access(&*state).await;
    let database = connection::get_database(&*state);
    let repo = TranscriptionRepository::new(database);
    repo.set_active(&request.transcription_id, &request.media_id).await?;
    Ok(())
}

/// List available providers and their status
#[utoipa::path(
    get,
    path = "/v1/transcription/providers",
    tag = "Transcription",
    responses(
        (status = 200, description = "List of providers", body = Vec<ProviderInfo>),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn list_providers(
    state: State<'_, crate::DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<Vec<ProviderInfo>> {
    let _guard = connection::with_db_access(&*state).await;
    let database = connection::get_database(&*state);
    
    let mut providers = Vec::new();
    
    // OpenAI
    let openai = OpenAIProvider::new(database.clone());
    let openai_status = openai.get_status().await;
    providers.push(ProviderInfo {
        name: "openai".to_string(),
        display_name: "OpenAI Whisper".to_string(),
        requires_api_key: true,
        requires_download: false,
        status: match openai_status {
            crate::transcription::provider::ProviderStatus::Ready => "ready".to_string(),
            crate::transcription::provider::ProviderStatus::NotConfigured => "not_configured".to_string(),
            crate::transcription::provider::ProviderStatus::Error { message } => {
                format!("error:{}", message)
            },
            _ => "not_configured".to_string(),
        },
        error_message: None,
    });
    
    // Groq
    let groq = GroqProvider::new(database.clone());
    let groq_status = groq.get_status().await;
    providers.push(ProviderInfo {
        name: "groq".to_string(),
        display_name: "Groq".to_string(),
        requires_api_key: true,
        requires_download: false,
        status: match groq_status {
            crate::transcription::provider::ProviderStatus::Ready => "ready".to_string(),
            crate::transcription::provider::ProviderStatus::NotConfigured => "not_configured".to_string(),
            crate::transcription::provider::ProviderStatus::Error { message } => {
                format!("error:{}", message)
            },
            _ => "not_configured".to_string(),
        },
        error_message: None,
    });
    
    // Local Whisper
    let local = LocalWhisperProvider::new(database.clone());
    let local_status = local.get_status().await;
    providers.push(ProviderInfo {
        name: "local-whisper".to_string(),
        display_name: "Local Whisper".to_string(),
        requires_api_key: false,
        requires_download: true,
        status: match local_status {
            crate::transcription::provider::ProviderStatus::Ready => "ready".to_string(),
            crate::transcription::provider::ProviderStatus::NotConfigured => "not_configured".to_string(),
            crate::transcription::provider::ProviderStatus::Error { message } => {
                format!("error:{}", message)
            },
            _ => "not_configured".to_string(),
        },
        error_message: None,
    });
    
    // Self-hosted
    let self_hosted = SelfHostedProvider::new(database.clone());
    let self_hosted_status = self_hosted.get_status().await;
    providers.push(ProviderInfo {
        name: "self-hosted".to_string(),
        display_name: "Self-Hosted".to_string(),
        requires_api_key: false,
        requires_download: false,
        status: match self_hosted_status {
            crate::transcription::provider::ProviderStatus::Ready => "ready".to_string(),
            crate::transcription::provider::ProviderStatus::NotConfigured => "not_configured".to_string(),
            crate::transcription::provider::ProviderStatus::Error { message } => {
                format!("error:{}", message)
            },
            _ => "not_configured".to_string(),
        },
        error_message: None,
    });
    
    Ok(providers)
}

/// Validate provider configuration
#[utoipa::path(
    post,
    path = "/v1/transcription/validate-provider",
    tag = "Transcription",
    params(
        ("providerName" = String, Path, description = "Provider name")
    ),
    responses(
        (status = 200, description = "Validation result", body = bool),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn validate_provider(
    state: State<'_, crate::DbState>,
    request_data: Option<ValidateProviderRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<bool> {
    let _guard = connection::with_db_access(&*state).await;
    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    let database = connection::get_database(&*state);
    
    let provider: Box<dyn TranscriptionProvider> = match request.provider_name.as_str() {
        "openai" => Box::new(OpenAIProvider::new(database.clone())),
        "groq" => Box::new(GroqProvider::new(database.clone())),
        "local-whisper" => Box::new(LocalWhisperProvider::new(database.clone())),
        "self-hosted" => Box::new(SelfHostedProvider::new(database.clone())),
        _ => return Err(AppError::BadRequest(format!("Unknown provider: {}", request.provider_name))),
    };
    
    provider.validate_config().await
        .map(|_| true)
        .map_err(|e| AppError::Internal(format!("Validation failed: {}", e)))
}

/// List available Whisper models
#[utoipa::path(
    get,
    path = "/v1/transcription/models",
    tag = "Transcription",
    responses(
        (status = 200, description = "List of models", body = Vec<ModelInfo>),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn list_available_models(
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<Vec<ModelInfo>> {
    let models = model_manager::list_available_models();
    let mut result = Vec::new();
    
    for model in models {
        let is_downloaded = model_manager::is_model_downloaded(&model.size)
            .unwrap_or(false);
        
        result.push(ModelInfo {
            name: model.name,
            size: model.size,
            file_size: model.file_size,
            download_url: model.download_url.unwrap_or_default(),
            is_downloaded,
        });
    }
    
    Ok(result)
}

/// Download a Whisper model
#[utoipa::path(
    post,
    path = "/v1/transcription/models/download",
    tag = "Transcription",
    params(
        ("modelSize" = String, Path, description = "Model size")
    ),
    responses(
        (status = 200, description = "Download status", body = String),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn download_model(
    _app: AppHandle,
    state: State<'_, crate::DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<ModelSizePathParams>,
) -> Result<String> {
    let _guard = connection::with_db_access(&*state).await;
    let model_size = path_params
        .and_then(|p| Some(p.model_size))
        .ok_or_else(|| AppError::BadRequest("Model size is required".to_string()))?;
    let database = connection::get_database(&*state);
    
    // Download with progress events
    // TODO: Implement progress events for Tauri 2
    model_manager::download_model(
        database,
        &model_size,
        None, // Progress callback disabled for now
    ).await?;
    
    Ok(format!("Model {} downloaded successfully", model_size))
}

/// Verify model integrity
#[utoipa::path(
    post,
    path = "/v1/transcription/models/verify",
    tag = "Transcription",
    params(
        ("modelSize" = String, Path, description = "Model size")
    ),
    responses(
        (status = 200, description = "Verification result", body = bool),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn verify_model(
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<ModelSizePathParams>,
) -> Result<bool> {
    let model_size = path_params
        .and_then(|p| Some(p.model_size))
        .ok_or_else(|| AppError::BadRequest("Model size is required".to_string()))?;
    model_manager::verify_model(&model_size)
}

/// Delete a downloaded model
#[utoipa::path(
    delete,
    path = "/v1/transcription/models/{modelSize}",
    tag = "Transcription",
    params(
        ("modelSize" = String, Path, description = "Model size")
    ),
    responses(
        (status = 200, description = "Model deleted successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn delete_model(
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<ModelSizePathParams>,
) -> Result<()> {
    let model_size = path_params
        .and_then(|p| Some(p.model_size))
        .ok_or_else(|| AppError::BadRequest("Model size is required".to_string()))?;
    model_manager::delete_model(&model_size)
}

