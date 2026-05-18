use crate::audio::preprocessing::compress_audio_bytes;
use crate::audio::read_audio_file;
use crate::db::repositories::TranscriptionRepository;
use crate::error::{AppError, Result};
use crate::transcription::provider::TranscriptionProvider;
use crate::transcription::providers::{
    GroqProvider, LocalWhisperProvider, OpenAIProvider, SelfHostedProvider,
};
use libsql::Database;
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::mpsc;

/// Transcription job
#[derive(Debug, Clone)]
pub struct TranscriptionJob {
    pub media_id: String,
    pub provider_name: String,
    pub transcription_id: String,
}

/// Transcription queue
pub struct TranscriptionQueue {
    sender: mpsc::Sender<TranscriptionJob>,
}

impl TranscriptionQueue {
    pub fn new(database: Arc<Database>, app_handle: AppHandle) -> Self {
        let (sender, mut receiver) = mpsc::channel::<TranscriptionJob>(100);

        // Spawn worker task
        let db_clone = database.clone();
        let app_clone = app_handle.clone();
        tokio::spawn(async move {
            while let Some(job) = receiver.recv().await {
                if let Err(e) =
                    process_transcription_job(job.clone(), db_clone.clone(), app_clone.clone())
                        .await
                {
                    tracing::error!("Failed to process transcription job: {}", e);
                }
            }
        });

        Self { sender }
    }

    pub async fn enqueue(&self, job: TranscriptionJob) -> Result<()> {
        self.sender
            .send(job)
            .await
            .map_err(|_| AppError::Internal("Queue is closed".to_string()))?;
        Ok(())
    }
}

/// Process a transcription job
async fn process_transcription_job(
    job: TranscriptionJob,
    database: Arc<Database>,
    _app_handle: AppHandle,
) -> Result<()> {
    let repo = TranscriptionRepository::new(database.clone());

    // Update status to processing
    repo.update_status(&job.transcription_id, "processing", None, None, None)
        .await?;

    // Emit status update
    // TODO: Fix event emission for Tauri 2
    // For now, events will be handled via polling or direct status checks
    tracing::info!(
        "Transcription processing: media_id={}, transcription_id={}",
        job.media_id,
        job.transcription_id
    );

    // Load original audio file
    let audio_data = read_audio_file(database.clone(), &job.media_id).await?;

    // Detect format from file
    let format =
        crate::audio::recorder::detect_format(&audio_data).unwrap_or_else(|| "webm".to_string());

    // Preprocess/compress audio
    let compressed_audio = compress_audio_bytes(&audio_data, &format)
        .await
        .map_err(|e| AppError::Internal(format!("Audio compression failed: {}", e)))?;

    // Get provider
    let provider: Box<dyn TranscriptionProvider> = match job.provider_name.as_str() {
        "openai" => {
            let mut p = OpenAIProvider::new(database.clone());
            p.initialize().await.map_err(|e| {
                AppError::Internal(format!("Provider initialization failed: {}", e))
            })?;
            Box::new(p)
        }
        "groq" => {
            let mut p = GroqProvider::new(database.clone());
            p.initialize().await.map_err(|e| {
                AppError::Internal(format!("Provider initialization failed: {}", e))
            })?;
            Box::new(p)
        }
        "local-whisper" => {
            let mut p = LocalWhisperProvider::new(database.clone());
            p.initialize().await.map_err(|e| {
                AppError::Internal(format!("Provider initialization failed: {}", e))
            })?;
            Box::new(p)
        }
        "self-hosted" => {
            let mut p = SelfHostedProvider::new(database.clone());
            p.initialize().await.map_err(|e| {
                AppError::Internal(format!("Provider initialization failed: {}", e))
            })?;
            Box::new(p)
        }
        _ => {
            return Err(AppError::BadRequest(format!(
                "Unknown provider: {}",
                job.provider_name
            )))
        }
    };

    // Transcribe
    let result = provider.transcribe(&compressed_audio, "mp3").await;

    match result {
        Ok(transcription_result) => {
            // Update database with result
            repo.update_status(
                &job.transcription_id,
                "complete",
                Some(transcription_result.text.clone()),
                transcription_result.confidence,
                None,
            )
            .await?;

            // Emit completion event
            // TODO: Fix event emission for Tauri 2
            tracing::info!(
                "Transcription complete: media_id={}, transcription_id={}",
                job.media_id,
                job.transcription_id
            );
        }
        Err(e) => {
            // Update database with error
            repo.update_status(&job.transcription_id, "failed", None, None, Some(e.clone()))
                .await?;

            // Emit error event
            // TODO: Fix event emission for Tauri 2
            tracing::error!(
                "Transcription failed: media_id={}, transcription_id={}, error={}",
                job.media_id,
                job.transcription_id,
                e
            );
        }
    }

    Ok(())
}
