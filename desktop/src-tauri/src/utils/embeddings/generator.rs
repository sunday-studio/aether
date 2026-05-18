use crate::error::{AppError, Result};
use crate::utils::embeddings::model_manager;
use fastembed::{EmbeddingModel as FastEmbeddingModel, InitOptions, TextEmbedding};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::OnceCell;

/// Global embedding model state (lazy-loaded)
static EMBEDDING_MODEL: OnceCell<Arc<Mutex<EmbeddingModel>>> = OnceCell::const_new();

/// Embedding model wrapper
enum EmbeddingModel {
    FastEmbed(TextEmbedding),
    LocalHashFallback,
}

impl EmbeddingModel {
    fn generate(&mut self, text: &str) -> Result<Vec<f32>> {
        if matches!(self, Self::LocalHashFallback)
            && model_manager::is_model_downloaded("all-MiniLM-L6-v2")?
        {
            *self = Self::FastEmbed(load_fastembed_model()?);
        }

        match self {
            Self::FastEmbed(model) => {
                let mut embeddings = model.embed(vec![text], None).map_err(|e| {
                    AppError::Internal(format!("Embedding generation failed: {}", e))
                })?;
                let vector = embeddings.pop().ok_or_else(|| {
                    AppError::Internal("Embedding model returned no vectors".to_string())
                })?;
                Ok(normalize_vector(vector))
            }
            Self::LocalHashFallback => Ok(generate_hash_embedding(text)),
        }
    }
}

/// Initialize the embedding model (lazy loading)
async fn init_model() -> Result<Arc<Mutex<EmbeddingModel>>> {
    if !model_manager::is_model_downloaded("all-MiniLM-L6-v2")? {
        tracing::warn!(
            "Local embedding model is not downloaded; using deterministic fallback embeddings"
        );
        return Ok(Arc::new(Mutex::new(EmbeddingModel::LocalHashFallback)));
    }

    let model = tokio::task::spawn_blocking(load_fastembed_model)
        .await
        .map_err(|e| AppError::Internal(format!("Embedding model load task failed: {}", e)))?
        .map_err(|e| AppError::Internal(format!("Failed to load embedding model: {}", e)))?;

    Ok(Arc::new(Mutex::new(EmbeddingModel::FastEmbed(model))))
}

fn load_fastembed_model() -> Result<TextEmbedding> {
    let cache_dir = model_manager::get_model_path("all-MiniLM-L6-v2")?;
    let mut options = InitOptions::new(FastEmbeddingModel::AllMiniLML6V2);
    options.cache_dir = cache_dir;
    options.show_download_progress = false;
    TextEmbedding::try_new(options)
        .map_err(|e| AppError::Internal(format!("Failed to load embedding model: {}", e)))
}

/// Generate embedding for text
/// This is the main public API
pub async fn generate_embedding(text: &str) -> Result<Vec<f32>> {
    if text.trim().is_empty() {
        return Err(AppError::BadRequest("Text cannot be empty".to_string()));
    }

    // Get or initialize model
    let model = EMBEDDING_MODEL.get_or_try_init(|| init_model()).await?;

    // Generate embedding in a blocking task (model inference may be CPU-intensive)
    let text = text.to_string();
    let model = Arc::clone(model);

    tokio::task::spawn_blocking(move || {
        let mut model = model
            .lock()
            .map_err(|_| AppError::Internal("Embedding model lock was poisoned".to_string()))?;
        model.generate(&text)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Embedding generation task failed: {}", e)))?
}

fn generate_hash_embedding(text: &str) -> Vec<f32> {
    let mut embedding = vec![0.0f32; 384];
    let hash = text
        .as_bytes()
        .iter()
        .fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64));

    for (index, value) in embedding.iter_mut().enumerate() {
        let seed = hash.wrapping_mul(index as u64 + 1);
        *value = ((seed % 2000) as f32 - 1000.0) / 1000.0;
    }

    normalize_vector(embedding)
}

fn normalize_vector(mut vector: Vec<f32>) -> Vec<f32> {
    let norm = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in &mut vector {
            *value /= norm;
        }
    }
    vector
}
