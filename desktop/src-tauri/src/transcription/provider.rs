use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Result of a transcription operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: Option<f32>,
    pub duration: Option<f32>,
}

/// Status of a transcription provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderStatus {
    Ready,
    NotConfigured,
    Downloading { progress: f32 },
    Error { message: String },
}

/// Trait for transcription providers
#[async_trait]
pub trait TranscriptionProvider: Send + Sync {
    /// Get the name of the provider
    fn name(&self) -> &str;
    
    /// Whether this provider requires an API key
    fn requires_api_key(&self) -> bool;
    
    /// Whether this provider requires a model download
    fn requires_download(&self) -> bool;
    
    /// Initialize the provider (load config, validate, etc.)
    async fn initialize(&mut self) -> Result<(), String>;
    
    /// Transcribe audio data
    async fn transcribe(&self, audio_data: &[u8], format: &str) -> Result<TranscriptionResult, String>;
    
    /// Get the current status of the provider
    async fn get_status(&self) -> ProviderStatus;
    
    /// Validate the provider configuration
    async fn validate_config(&self) -> Result<(), String>;
}
