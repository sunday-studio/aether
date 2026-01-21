use crate::api::openapi;
use std::path::PathBuf;

fn main() {
    let spec_json = openapi::get_openapi_json();
    
    // Determine the workspace root (go up from src-tauri to desktop)
    let mut output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    output_path.pop(); 
    output_path.push("src");
    output_path.push("openapi");
    output_path.push("spec.json");
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create docs directory");
    }
    
    std::fs::write(&output_path, spec_json).expect("Failed to write OpenAPI spec");
    
    println!("OpenAPI spec generated at: {}", output_path.display());
}
