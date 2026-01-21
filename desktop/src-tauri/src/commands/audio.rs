use crate::audio::{delete_audio, read_audio_file, save_audio_file};
use crate::db::connection;
use crate::error::{AppError, Result};
use crate::settings;
use tauri::{AppHandle, State};

/// Save audio recording to filesystem and database
#[tauri::command]
pub async fn save_audio_recording(
    _app: AppHandle,
    state: State<'_, crate::DbState>,
    entry_id: String,
    audio_data: Vec<u8>,
    duration: f32,
    format: String,
    auto_transcribe: Option<bool>,
) -> Result<String> {
    if entry_id.is_empty() {
        return Err(AppError::BadRequest("Entry ID is required".to_string()));
    }

    let database = connection::get_database(&*state);
    
    // Save audio file
    let media_id = save_audio_file(
        database.clone(),
        entry_id,
        audio_data,
        duration,
        format,
    ).await?;

    // Optionally queue transcription
    let should_transcribe = auto_transcribe.unwrap_or_else(|| {
        // Check settings for auto-transcribe
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            settings::get_setting(database.clone(), "transcription.auto_transcribe")
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| "false".to_string())
                == "true"
        })
    });

    if should_transcribe {
        // Get default provider
        let _default_provider = settings::get_setting(database.clone(), "transcription.default_provider")
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
#[tauri::command]
pub async fn get_audio_data(
    state: State<'_, crate::DbState>,
    media_id: String,
) -> Result<Vec<u8>> {
    if media_id.is_empty() {
        return Err(AppError::BadRequest("Media ID is required".to_string()));
    }

    let database = connection::get_database(&*state);
    read_audio_file(database, &media_id).await
}

/// Delete audio recording
#[tauri::command]
pub async fn delete_audio_recording(
    state: State<'_, crate::DbState>,
    media_id: String,
) -> Result<()> {
    if media_id.is_empty() {
        return Err(AppError::BadRequest("Media ID is required".to_string()));
    }

    let database = connection::get_database(&*state);
    delete_audio(database, &media_id).await
}
