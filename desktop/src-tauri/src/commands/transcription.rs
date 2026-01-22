use crate::db::connection;
use crate::db::repositories::TranscriptionRepository;
use crate::error::{AppError, Result};
use crate::settings;
use crate::transcription::model_manager;
use crate::transcription::providers::{GroqProvider, LocalWhisperProvider, OpenAIProvider, SelfHostedProvider};
use crate::transcription::provider::TranscriptionProvider;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use utoipa::ToSchema;

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
    media_id: String,
    provider_name: Option<String>,
) -> Result<String> {
    if media_id.is_empty() {
        return Err(AppError::BadRequest("Media ID is required".to_string()));
    }

    let database = connection::get_database(&*state);
    
    // Get provider name (default or specified)
    let provider = provider_name.unwrap_or_else(|| {
        // Get from settings
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            settings::get_setting(database.clone(), "transcription.default_provider")
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| "openai".to_string())
        })
    });

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
        ("mediaId" = String, Path, description = "Media ID")
    ),
    responses(
        (status = 200, description = "List of transcriptions", body = Vec<crate::db::models::AudioTranscription>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_transcriptions(
    state: State<'_, crate::DbState>,
    media_id: String,
) -> Result<Vec<crate::db::models::AudioTranscription>> {
    if media_id.is_empty() {
        return Err(AppError::BadRequest("Media ID is required".to_string()));
    }

    let database = connection::get_database(&*state);
    let repo = TranscriptionRepository::new(database);
    repo.find_by_media_id(&media_id).await
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
        (status = 200, description = "Transcription", body = Option<crate::db::models::AudioTranscription>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_transcription_by_id(
    state: State<'_, crate::DbState>,
    transcription_id: String,
) -> Result<Option<crate::db::models::AudioTranscription>> {
    if transcription_id.is_empty() {
        return Err(AppError::BadRequest("Transcription ID is required".to_string()));
    }

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
    transcription_id: String,
    media_id: String,
) -> Result<()> {
    if transcription_id.is_empty() || media_id.is_empty() {
        return Err(AppError::BadRequest("Transcription ID and Media ID are required".to_string()));
    }

    let database = connection::get_database(&*state);
    let repo = TranscriptionRepository::new(database);
    repo.set_active(&transcription_id, &media_id).await?;
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
) -> Result<Vec<ProviderInfo>> {
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
    provider_name: String,
) -> Result<bool> {
    let database = connection::get_database(&*state);
    
    let provider: Box<dyn TranscriptionProvider> = match provider_name.as_str() {
        "openai" => Box::new(OpenAIProvider::new(database.clone())),
        "groq" => Box::new(GroqProvider::new(database.clone())),
        "local-whisper" => Box::new(LocalWhisperProvider::new(database.clone())),
        "self-hosted" => Box::new(SelfHostedProvider::new(database.clone())),
        _ => return Err(AppError::BadRequest(format!("Unknown provider: {}", provider_name))),
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
pub async fn list_available_models() -> Result<Vec<ModelInfo>> {
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
    model_size: String,
) -> Result<String> {
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
pub async fn verify_model(model_size: String) -> Result<bool> {
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
pub async fn delete_model(model_size: String) -> Result<()> {
    model_manager::delete_model(&model_size)
}

/// Get a setting value
#[tauri::command]
pub async fn get_setting(
    state: State<'_, crate::DbState>,
    key: String,
) -> Result<Option<String>> {
    let database = connection::get_database(&*state);
    settings::get_setting(database, &key).await
}

/// Set a setting value
#[tauri::command]
pub async fn set_setting(
    state: State<'_, crate::DbState>,
    key: String,
    value: String,
) -> Result<()> {
    let database = connection::get_database(&*state);
    settings::set_setting(database, &key, &value).await
}
