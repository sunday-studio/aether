pub mod generator;
pub mod model_manager;

pub use generator::generate_embedding;
pub use model_manager::{
    delete_model, download_model, ensure_models_directory, get_model_path, is_model_downloaded,
    list_available_models, verify_model, ModelInfo,
};
