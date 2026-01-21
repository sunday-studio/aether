use crate::settings;
use crate::transcription::provider::{ProviderStatus, TranscriptionProvider, TranscriptionResult};
use async_trait::async_trait;
use libsql::Database;
use std::path::PathBuf;
use std::sync::Arc;

pub struct LocalWhisperProvider {
    database: Arc<Database>,
    model_path: Option<PathBuf>,
    model_size: Option<String>,
    initialized: bool,
}

impl LocalWhisperProvider {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            database,
            model_path: None,
            model_size: None,
            initialized: false,
        }
    }

    fn get_models_directory() -> PathBuf {
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Aether")
                .join("models")
        }

        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("aether")
                .join("models")
        }

        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(appdata)
                .join("Aether")
                .join("models")
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            PathBuf::from(".").join("models")
        }
    }
}

#[async_trait]
impl TranscriptionProvider for LocalWhisperProvider {
    fn name(&self) -> &str {
        "local-whisper"
    }

    fn requires_api_key(&self) -> bool {
        false
    }

    fn requires_download(&self) -> bool {
        true
    }

    async fn initialize(&mut self) -> Result<(), String> {
        // Get model size and path from settings
        let model_size = settings::get_setting(self.database.clone(), "transcription.local_whisper.model_size")
            .await
            .map_err(|e| format!("Failed to get model size: {}", e))?
            .unwrap_or_else(|| "base".to_string());

        let models_dir = Self::get_models_directory();
        let model_path = models_dir.join(format!("ggml-{}.bin", model_size));

        if !model_path.exists() {
            return Err(format!("Model not found at {:?}. Please download the model first.", model_path));
        }

        self.model_path = Some(model_path);
        self.model_size = Some(model_size);
        self.initialized = true;
        Ok(())
    }

    async fn transcribe(&self, _audio_data: &[u8], _format: &str) -> Result<TranscriptionResult, String> {
        if !self.initialized {
            return Err("Provider not initialized".to_string());
        }

        let model_path = self.model_path.as_ref().ok_or("Model path not set")?;

        // TODO: Implement actual Whisper transcription using whisper-rs or candle
        // For now, return an error indicating it's not yet implemented
        Err(format!(
            "Local Whisper transcription not yet implemented. Model path: {:?}",
            model_path
        ))
    }

    async fn get_status(&self) -> ProviderStatus {
        let model_size = match settings::get_setting(self.database.clone(), "transcription.local_whisper.model_size").await {
            Ok(Some(size)) => size,
            _ => return ProviderStatus::NotConfigured,
        };

        let models_dir = Self::get_models_directory();
        let model_path = models_dir.join(format!("ggml-{}.bin", model_size));

        if model_path.exists() {
            ProviderStatus::Ready
        } else {
            ProviderStatus::NotConfigured
        }
    }

    async fn validate_config(&self) -> Result<(), String> {
        let model_size = settings::get_setting(self.database.clone(), "transcription.local_whisper.model_size")
            .await
            .map_err(|e| format!("Failed to get model size: {}", e))?
            .ok_or_else(|| "Model size not configured".to_string())?;

        let models_dir = Self::get_models_directory();
        let model_path = models_dir.join(format!("ggml-{}.bin", model_size));

        if model_path.exists() {
            Ok(())
        } else {
            Err(format!("Model not found at {:?}", model_path))
        }
    }
}
