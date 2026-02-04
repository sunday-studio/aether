use desktop_lib::api::openapi;
use std::path::PathBuf;

fn main() {
    let spec_json = openapi::get_openapi_json();

    // CARGO_MANIFEST_DIR is src-tauri/tools; go up to src-tauri then to desktop
    let mut desktop_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    desktop_root.pop(); // src-tauri
    desktop_root.pop(); // desktop

    let mut desktop_spec = desktop_root.clone();
    desktop_spec.push("src");
    desktop_spec.push("openapi");
    desktop_spec.push("spec.json");

    for output_path in [&desktop_spec] {
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create openapi directory");
        }
        std::fs::write(output_path, &spec_json).expect("Failed to write OpenAPI spec");
        println!("OpenAPI spec generated at: {}", output_path.display());
    }
}
