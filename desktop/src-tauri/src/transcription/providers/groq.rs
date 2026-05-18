use crate::settings;
use crate::transcription::provider::{ProviderStatus, TranscriptionProvider, TranscriptionResult};
use async_trait::async_trait;
use libsql::Database;
use reqwest::multipart;
use std::sync::Arc;

pub struct GroqProvider {
    database: Arc<Database>,
    api_key: Option<String>,
    initialized: bool,
}

impl GroqProvider {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            database,
            api_key: None,
            initialized: false,
        }
    }
}

#[async_trait]
impl TranscriptionProvider for GroqProvider {
    fn name(&self) -> &str {
        "groq"
    }

    fn requires_api_key(&self) -> bool {
        true
    }

    fn requires_download(&self) -> bool {
        false
    }

    async fn initialize(&mut self) -> Result<(), String> {
        let api_key = settings::get_setting(self.database.clone(), "transcription.groq.api_key")
            .await
            .map_err(|e| format!("Failed to get API key: {}", e))?
            .ok_or_else(|| "API key not configured".to_string())?;

        self.api_key = Some(api_key);
        self.initialized = true;
        Ok(())
    }

    async fn transcribe(
        &self,
        audio_data: &[u8],
        format: &str,
    ) -> Result<TranscriptionResult, String> {
        if !self.initialized {
            return Err("Provider not initialized".to_string());
        }

        let api_key = self.api_key.as_ref().ok_or("API key not set")?;

        let client = reqwest::Client::new();

        // Create multipart form (Groq uses same API as OpenAI)
        let form = multipart::Form::new()
            .text("model", "whisper-large-v3")
            .text("language", "en")
            .part(
                "file",
                multipart::Part::bytes(audio_data.to_vec())
                    .file_name(format!("audio.{}", format))
                    .mime_str(&get_mime_type(format))
                    .map_err(|e| format!("Failed to set MIME type: {}", e))?,
            );

        let response = client
            .post("https://api.groq.com/openai/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", api_key))
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("API error ({}): {}", status, error_text));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let text = result["text"]
            .as_str()
            .ok_or("Missing text in response")?
            .to_string();

        Ok(TranscriptionResult {
            text,
            confidence: None,
            duration: None,
        })
    }

    async fn get_status(&self) -> ProviderStatus {
        if let Ok(Some(_)) =
            settings::get_setting(self.database.clone(), "transcription.groq.api_key").await
        {
            ProviderStatus::Ready
        } else {
            ProviderStatus::NotConfigured
        }
    }

    async fn validate_config(&self) -> Result<(), String> {
        let api_key = settings::get_setting(self.database.clone(), "transcription.groq.api_key")
            .await
            .map_err(|e| format!("Failed to get API key: {}", e))?
            .ok_or_else(|| "API key not configured".to_string())?;

        // Test with a minimal request
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.groq.com/openai/v1/models")
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| format!("Validation request failed: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Invalid API key (status: {})", response.status()))
        }
    }
}

fn get_mime_type(format: &str) -> &str {
    match format.to_lowercase().as_str() {
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "webm" => "audio/webm",
        "m4a" => "audio/mp4",
        _ => "audio/mpeg",
    }
}
