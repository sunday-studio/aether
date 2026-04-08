use crate::audio::{delete_audio, read_audio_file, save_audio_file};
use crate::commands::params::{
    EmptyPathParams, EmptyQueryParams, EntryIdPathParams, MediaIdPathParams,
};
use crate::db::connection;
use crate::db::repositories::MediaRepository;
use crate::error::{AppError, Result};
use crate::settings;
use crate::sync;
use serde::Deserialize;
use std::sync::Arc;
use tauri::{AppHandle, State};
use utoipa::ToSchema;

/// Request to save audio recording
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SaveAudioRecordingRequest {
    pub entry_id: String,
    pub audio_data: Vec<u8>,
    pub duration: f32,
    pub format: String,
    #[serde(default)]
    pub auto_transcribe: Option<bool>,
}

/// Save audio recording to filesystem and database
#[utoipa::path(
    post,
    path = "/v1/audio",
    tag = "Audio",
    request_body(content = serde_json::Value, description = "Audio recording data"),
    params(
        ("entryId" = String, Path, description = "Entry ID"),
    ),
    responses(
        (status = 200, description = "Media ID of saved audio", body = String),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn save_audio_recording(
    _app: AppHandle,
    state: State<'_, crate::DbState>,
    request_data: Option<SaveAudioRecordingRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<String> {
    let _guard = connection::with_db_access(&*state).await;
    let request =
        request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    if request.entry_id.is_empty() {
        return Err(AppError::BadRequest("Entry ID is required".to_string()));
    }

    let database = connection::get_database(&*state);

    // Save audio file
    let media_id = save_audio_file(
        database.clone(),
        request.entry_id,
        request.audio_data,
        request.duration,
        request.format,
    )
    .await?;

    // Optionally queue transcription
    let should_transcribe = if let Some(should) = request.auto_transcribe {
        should
    } else {
        // Check settings for auto-transcribe
        settings::get_setting(database.clone(), "transcription.auto_transcribe")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "false".to_string())
            == "true"
    };

    if should_transcribe {
        // Get default provider
        let _default_provider =
            settings::get_setting(database.clone(), "transcription.default_provider")
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| "openai".to_string());

        // TODO: Auto-queue transcription when auto_transcribe is enabled
        // For now, user can manually trigger transcription
    }

    Ok(media_id)
}

/// Get audio file data
#[utoipa::path(
    get,
    path = "/v1/audio/{mediaId}",
    tag = "Audio",
    params(
        ("mediaId" = String, Path, description = "Media ID")
    ),
    responses(
        (status = 200, description = "Audio file data", body = Vec<u8>),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Audio not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_audio_data(
    state: State<'_, crate::DbState>,
    engine: State<'_, Arc<sync::SyncEngine>>,
    _request_data: Option<crate::commands::params::EmptyRequest>,
    _query_params: Option<crate::commands::params::EmptyQueryParams>,
    path_params: Option<MediaIdPathParams>,
) -> Result<Vec<u8>> {
    let _guard = connection::with_db_access(&*state).await;
    let media_id = path_params
        .and_then(|p| Some(p.media_id))
        .ok_or_else(|| AppError::BadRequest("Media ID is required".to_string()))?;
    if media_id.is_empty() {
        return Err(AppError::BadRequest("Media ID is required".to_string()));
    }

    let database = connection::get_database(&*state);
    let url = engine.try_get_url();
    let key = engine.try_get_key().await;
    let device_id = engine.get_device_id().await.ok();
    let device_token = engine.get_device_token().await.ok().flatten();
    let policy = settings::get_setting(database.clone(), "sync.media_sync_policy")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "on_demand".to_string());
    let _ = sync::media::ensure_media_blob(
        database.as_ref(),
        &media_id,
        url.as_deref(),
        key.as_ref(),
        device_id.as_deref(),
        device_token.as_deref(),
        &policy,
    )
    .await;
    read_audio_file(database, &media_id).await
}

/// Delete audio recording
#[utoipa::path(
    delete,
    path = "/v1/audio/{mediaId}",
    tag = "Audio",
    params(
        ("mediaId" = String, Path, description = "Media ID")
    ),
    responses(
        (status = 200, description = "Audio deleted successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn delete_audio_recording(
    state: State<'_, crate::DbState>,
    _request_data: Option<crate::commands::params::EmptyRequest>,
    _query_params: Option<crate::commands::params::EmptyQueryParams>,
    path_params: Option<MediaIdPathParams>,
) -> Result<()> {
    let _guard = connection::with_db_access(&*state).await;
    let media_id = path_params
        .and_then(|p| Some(p.media_id))
        .ok_or_else(|| AppError::BadRequest("Media ID is required".to_string()))?;
    if media_id.is_empty() {
        return Err(AppError::BadRequest("Media ID is required".to_string()));
    }

    let database = connection::get_database(&*state);
    delete_audio(database, &media_id).await
}

/// Get all media items (audio, images, etc.) for an entry
#[utoipa::path(
    get,
    path = "/v1/entry/{entryId}/media",
    tag = "Audio",
    params(
        ("entryId" = String, Path, description = "Entry ID")
    ),
    responses(
        (status = 200, description = "List of media items", body = Vec<MediaItem>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_media_items_for_entry(
    state: State<'_, crate::DbState>,
    _request_data: Option<crate::commands::params::EmptyRequest>,
    _query_params: Option<crate::commands::params::EmptyQueryParams>,
    path_params: Option<EntryIdPathParams>,
) -> Result<Vec<crate::db::models::MediaItem>> {
    let _guard = connection::with_db_access(&*state).await;
    let entry_id = path_params
        .and_then(|p| Some(p.entry_id))
        .ok_or_else(|| AppError::BadRequest("Entry ID is required".to_string()))?;
    if entry_id.is_empty() {
        return Err(AppError::BadRequest("Entry ID is required".to_string()));
    }

    let database = connection::get_database(&*state);
    let repo = MediaRepository::new(database);
    repo.find_by_entry_id(&entry_id).await
}

/// Get audio metadata without loading the full file
#[utoipa::path(
    get,
    path = "/v1/audio/{mediaId}/metadata",
    tag = "Audio",
    params(
        ("mediaId" = String, Path, description = "Media ID")
    ),
    responses(
        (status = 200, description = "Audio metadata", body = Option<MediaItem>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_audio_metadata(
    state: State<'_, crate::DbState>,
    _request_data: Option<crate::commands::params::EmptyRequest>,
    _query_params: Option<crate::commands::params::EmptyQueryParams>,
    path_params: Option<MediaIdPathParams>,
) -> Result<Option<crate::db::models::MediaItem>> {
    let _guard = connection::with_db_access(&*state).await;
    let media_id = path_params
        .and_then(|p| Some(p.media_id))
        .ok_or_else(|| AppError::BadRequest("Media ID is required".to_string()))?;
    if media_id.is_empty() {
        return Err(AppError::BadRequest("Media ID is required".to_string()));
    }

    let database = connection::get_database(&*state);
    let repo = MediaRepository::new(database);
    repo.find_by_id(&media_id).await
}
