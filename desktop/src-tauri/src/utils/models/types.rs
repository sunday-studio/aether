/// Model category for organizing models
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelCategory {
    Transcription,
    Embedding,
}

/// Unified model information structure
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub size: String,
    pub file_size: u64,
    pub download_url: Option<String>,
    pub checksum: Option<String>,
    pub is_downloaded: bool,
    // Model-specific fields
    pub dimensions: Option<u32>, // Some for embeddings
    pub category: ModelCategory,
}
