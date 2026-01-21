pub mod generator;
pub mod model_manager;

pub use generator::generate_embedding;
pub use model_manager::{get_model_path, list_available_models, ModelInfo};

/// Generate embedding for text using local model
/// Returns a 384-dimensional vector (F32)
pub async fn generate_embedding(text: &str) -> crate::error::Result<Vec<f32>> {
    generator::generate_embedding(text).await
}
