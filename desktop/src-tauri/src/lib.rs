pub mod api;
pub mod audio;
pub mod commands;
pub mod db;
pub mod error;
pub mod handlers;
pub mod settings;
pub mod transcription;
pub mod utils;

pub use db::DbState;
pub use error::{AppError, Result};

use commands::{
    activity, entry, goal, sync, tag, task, trash, search,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    // Default: info level globally, debug for our crate (includes activity logging)
                    // You can override with RUST_LOG env var, e.g.:
                    // RUST_LOG=debug bun tauri dev
                    tracing_subscriber::EnvFilter::new("info,desktop_lib=debug")
                }),
        )
        .with_target(true) // Show target module path for better debugging
        .with_thread_ids(false) // Don't show thread IDs
        .with_line_number(true) // Show line numbers
        .init();

    tracing::info!("Tauri application starting...");

    let mut builder = tauri::Builder::default();

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_plugin_macos_haptics::init());
    }

    // Initialize database and run migrations
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let db_state = rt.block_on(async {
        // Load environment variables
        dotenvy::dotenv().ok();

        // Initialize database
        let state = db::initialize().await
            .expect("Failed to initialize database");
        
        // Run migrations
        let database = db::connection::get_database(&state);
        db::migrations::run_migrations(&database).await
            .expect("Failed to run migrations");
        
        // Ensure media directory exists
        audio::ensure_media_directory()
            .expect("Failed to create media directory");
        
        state
    });

    builder
        .manage(db_state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Tag commands
            tag::get_all_tags,
            tag::create_tag,
            tag::bulk_create_tags,
            // Entry commands
            entry::get_entries,
            entry::get_entry_by_id,
            entry::create_entry,
            entry::bulk_create_entries,
            entry::update_entry,
            entry::delete_entry,
            entry::add_tags_to_entry,
            entry::remove_tags_from_entry,
            // Task commands
            task::create_task,
            task::get_inbox_tasks,
            task::get_overdue_tasks,
            task::get_task_by_id,
            task::update_task,
            task::delete_task,
            task::get_subtasks,
            task::create_subtask,
            task::update_subtask,
            task::delete_subtask,
            task::reorder_subtasks,
            task::add_tags_to_task,
            task::remove_tags_from_task,
            task::add_goal_to_task,
            task::remove_goal_from_task,
            // Goal commands
            goal::get_goals,
            goal::get_goal_by_id,
            goal::create_goal,
            goal::update_goal,
            goal::delete_goal,
            goal::get_goal_instances,
            goal::get_current_goal_instance,
            goal::add_tags_to_goal,
            goal::remove_tags_from_goal,
            // Trash commands
            trash::get_trashed_tasks,
            trash::restore_task,
            // Sync commands
            sync::configure_sync,
            sync::sync,
            // Activity commands
            activity::get_activities,
            // Search commands
            search::search_resources,
            // Audio commands
            commands::audio::save_audio_recording,
            commands::audio::get_audio_data,
            commands::audio::delete_audio_recording,
            // Transcription commands
            commands::transcription::start_transcription,
            commands::transcription::get_transcriptions,
            commands::transcription::set_active_transcription,
            commands::transcription::list_providers,
            commands::transcription::validate_provider,
            commands::transcription::list_available_models,
            commands::transcription::download_model,
            commands::transcription::verify_model,
            commands::transcription::delete_model,
            commands::transcription::get_setting,
            commands::transcription::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
