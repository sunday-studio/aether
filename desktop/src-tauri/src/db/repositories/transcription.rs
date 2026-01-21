use crate::db::models::AudioTranscription;
use crate::error::{AppError, Result};
use crate::utils::generate_id;
use chrono::Utc;
use libsql::Database;
use std::sync::Arc;

pub struct TranscriptionRepository {
    database: Arc<Database>,
}

impl TranscriptionRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Create a new transcription
    pub async fn create(
        &self,
        media_id: String,
        provider: String,
        provider_config: Option<serde_json::Value>,
    ) -> Result<AudioTranscription> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let id = generate_id("transcription");
        let now = Utc::now();
        let created_at_str = now.to_rfc3339();
        let provider_config_str = provider_config
            .as_ref()
            .map(|c| serde_json::to_string(c))
            .transpose()
            .map_err(|e| AppError::Serialization(e))?;

        conn.execute(
            "INSERT INTO audio_transcriptions (id, media_id, transcription_text, provider, provider_config, status, is_active, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            libsql::params![
                id.clone(),
                media_id.clone(),
                "", // Empty text initially
                provider.clone(),
                provider_config_str,
                "pending",
                0, // Not active initially
                created_at_str
            ],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(AudioTranscription {
            id,
            media_id,
            transcription_text: String::new(),
            provider,
            provider_config,
            confidence_score: None,
            status: "pending".to_string(),
            error_message: None,
            is_active: false,
            created_at: now,
        })
    }

    /// Get transcription by ID
    pub async fn find_by_id(&self, id: &str) -> Result<Option<AudioTranscription>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT id, media_id, transcription_text, provider, provider_config, confidence_score, 
                        status, error_message, is_active, created_at 
                 FROM audio_transcriptions 
                 WHERE id = ?1",
                libsql::params![id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        if let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            Ok(Some(self.row_to_transcription(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get all transcriptions for a media item
    pub async fn find_by_media_id(&self, media_id: &str) -> Result<Vec<AudioTranscription>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT id, media_id, transcription_text, provider, provider_config, confidence_score, 
                        status, error_message, is_active, created_at 
                 FROM audio_transcriptions 
                 WHERE media_id = ?1 
                 ORDER BY created_at ASC",
                libsql::params![media_id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut transcriptions = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            transcriptions.push(self.row_to_transcription(row)?);
        }

        Ok(transcriptions)
    }

    /// Set a transcription as active (and deactivate others for the same media)
    pub async fn set_active(&self, transcription_id: &str, media_id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // First, deactivate all transcriptions for this media
        conn.execute(
            "UPDATE audio_transcriptions SET is_active = 0 WHERE media_id = ?1",
            libsql::params![media_id],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        // Then activate the specified transcription
        conn.execute(
            "UPDATE audio_transcriptions SET is_active = 1 WHERE id = ?1",
            libsql::params![transcription_id],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Update transcription status and result
    pub async fn update_status(
        &self,
        id: &str,
        status: &str,
        transcription_text: Option<String>,
        confidence_score: Option<f32>,
        error_message: Option<String>,
    ) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        if let Some(text) = transcription_text {
            if let Some(score) = confidence_score {
                conn.execute(
                    "UPDATE audio_transcriptions 
                     SET status = ?1, transcription_text = ?2, confidence_score = ?3, error_message = ?4 
                     WHERE id = ?5",
                    libsql::params![status, text, score, error_message, id],
                )
                .await
                .map_err(|e| AppError::LibSQL(e))?;
            } else {
                conn.execute(
                    "UPDATE audio_transcriptions 
                     SET status = ?1, transcription_text = ?2, error_message = ?3 
                     WHERE id = ?4",
                    libsql::params![status, text, error_message, id],
                )
                .await
                .map_err(|e| AppError::LibSQL(e))?;
            }
        } else {
            conn.execute(
                "UPDATE audio_transcriptions 
                 SET status = ?1, error_message = ?2 
                 WHERE id = ?3",
                libsql::params![status, error_message, id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;
        }

        Ok(())
    }

    /// Helper to convert database row to AudioTranscription
    fn row_to_transcription(&self, row: libsql::Row) -> Result<AudioTranscription> {
        let id: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
        let media_id: String = row.get(1).map_err(|e| AppError::LibSQL(e))?;
        let transcription_text: String = row.get(2).map_err(|e| AppError::LibSQL(e))?;
        let provider: String = row.get(3).map_err(|e| AppError::LibSQL(e))?;
        let provider_config_str: Option<String> = row.get(4).map_err(|e| AppError::LibSQL(e))?;
        let confidence_score: Option<f64> = row.get(5).map_err(|e| AppError::LibSQL(e))?;
        let status: String = row.get(6).map_err(|e| AppError::LibSQL(e))?;
        let error_message: Option<String> = row.get(7).map_err(|e| AppError::LibSQL(e))?;
        let is_active: i64 = row.get(8).map_err(|e| AppError::LibSQL(e))?;
        let created_at_str: String = row.get(9).map_err(|e| AppError::LibSQL(e))?;

        let provider_config = provider_config_str
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e| AppError::Serialization(e))?;
        
        let confidence_score_f32 = confidence_score.map(|s| s as f32);
        
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&Utc);

        Ok(AudioTranscription {
            id,
            media_id,
            transcription_text,
            provider,
            provider_config,
            confidence_score: confidence_score_f32,
            status,
            error_message,
            is_active: is_active != 0,
            created_at,
        })
    }
}
