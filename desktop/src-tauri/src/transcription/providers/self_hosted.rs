use crate::settings;
use crate::transcription::provider::{ProviderStatus, TranscriptionProvider, TranscriptionResult};
use async_trait::async_trait;
use libsql::Database;
use reqwest::multipart;
use std::sync::Arc;

pub struct SelfHostedProvider {
    database: Arc<Database>,
    endpoint: Option<String>,
    auth_token: Option<String>,
    initialized: bool,
}

impl SelfHostedProvider {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            database,
            endpoint: None,
            auth_token: None,
            initialized: false,
        }
    }
}

#[async_trait]
impl TranscriptionProvider for SelfHostedProvider {
    fn name(&self) -> &str {
        "self-hosted"
    }

    fn requires_api_key(&self) -> bool {
        false // Optional auth token
    }

    fn requires_download(&self) -> bool {
        false
    }

    async fn initialize(&mut self) -> Result<(), String> {
        let endpoint = settings::get_setting(self.database.clone(), "transcription.self_hosted.endpoint")
            .await
            .map_err(|e| format!("Failed to get endpoint: {}", e))?
            .ok_or_else(|| "Endpoint not configured".to_string())?;

        let auth_token = settings::get_setting(self.database.clone(), "transcription.self_hosted.auth_token")
            .await
            .map_err(|e| format!("Failed to get auth token: {}", e))
            .ok()
            .flatten();

        self.endpoint = Some(endpoint);
        self.auth_token = auth_token;
        self.initialized = true;
        Ok(())
    }

    async fn transcribe(&self, audio_data: &[u8], format: &str) -> Result<TranscriptionResult, String> {
        if !self.initialized {
            return Err("Provider not initialized".to_string());
        }

        let endpoint = self.endpoint.as_ref().ok_or("Endpoint not set")?;
        let url = format!("{}/transcribe", endpoint.trim_end_matches('/'));

        let client = reqwest::Client::new();
        
        // Create multipart form
        let mut form = multipart::Form::new()
            .part("audio", multipart::Part::bytes(audio_data.to_vec())
                .file_name(format!("audio.{}", format))
                .mime_str(&get_mime_type(format))
                .map_err(|e| format!("Failed to set MIME type: {}", e))?);

        let mut request = client.post(&url);

        // Add auth token if available
        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("API error ({}): {}", response.status(), error_text));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let text = result["text"]
            .as_str()
            .ok_or("Missing text in response")?
            .to_string();

        let confidence = result["confidence"]
            .as_f64()
            .map(|c| c as f32);

        let duration = result["duration"]
            .as_f64()
            .map(|d| d as f32);

        Ok(TranscriptionResult {
            text,
            confidence,
            duration,
        })
    }

    async fn get_status(&self) -> ProviderStatus {
        let endpoint = match settings::get_setting(self.database.clone(), "transcription.self_hosted.endpoint").await {
            Ok(Some(e)) => e,
            _ => return ProviderStatus::NotConfigured,
        };

        // Try health check
        let client = reqwest::Client::new();
        let health_url = format!("{}/health", endpoint.trim_end_matches('/'));
        
        match client.get(&health_url).send().await {
            Ok(response) if response.status().is_success() => ProviderStatus::Ready,
            Ok(_) => ProviderStatus::Error {
                message: "Health check failed".to_string(),
            },
            Err(e) => ProviderStatus::Error {
                message: format!("Connection failed: {}", e),
            },
        }
    }

    async fn validate_config(&self) -> Result<(), String> {
        let endpoint = settings::get_setting(self.database.clone(), "transcription.self_hosted.endpoint")
            .await
            .map_err(|e| format!("Failed to get endpoint: {}", e))?
            .ok_or_else(|| "Endpoint not configured".to_string())?;

        // Test health check endpoint
        let client = reqwest::Client::new();
        let health_url = format!("{}/health", endpoint.trim_end_matches('/'));
        
        let response = client
            .get(&health_url)
            .send()
            .await
            .map_err(|e| format!("Health check failed: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Health check failed (status: {})", response.status()))
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
