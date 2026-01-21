pub mod generator;
pub mod model_manager;

pub use generator::generate_embedding;
pub use model_manager::{
    get_model_path, list_available_models, ModelInfo,
    download_model, verify_model, delete_model, is_model_downloaded,
    ensure_models_directory,
};
