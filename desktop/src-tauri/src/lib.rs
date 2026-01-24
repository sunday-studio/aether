pub mod api;
pub mod audio;
pub mod commands;
pub mod db;
pub mod error;
pub mod handlers;
pub mod media;
pub mod settings;
pub mod sync;
pub mod transcription;
pub mod utils;

pub use db::DbState;
pub use error::{AppError, Result};

use commands::{
    activity, canvas, entry, goal, tag, task, trash, search, bookmark, link,
    sync as sync_commands,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Manager, WindowEvent};

/// Tracks whether the main window has focus. Used by periodic sync to run only when focused.
pub struct WindowFocus(pub Arc<AtomicBool>);

impl WindowFocus {
    pub fn set(&self, focused: bool) {
        self.0.store(focused, Ordering::Relaxed);
    }
    pub fn get(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

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
        media::ensure_media_directory()
            .expect("Failed to create media directory");
        
        state
    });

    let sync_engine = Arc::new(sync::SyncEngine::new(db_state.clone()));
    let _ = rt.block_on(sync_engine.hydrate_from_metadata());
    let window_focus = WindowFocus(Arc::new(AtomicBool::new(true)));

    builder
        .manage(db_state)
        .manage(sync_engine.clone())
        .manage(window_focus)
        .on_window_event(|window, event| {
            match event {
                WindowEvent::CloseRequested { .. } => {
                    if let Some(engine) = window.app_handle().try_state::<Arc<sync::SyncEngine>>() {
                        let _ = tauri::async_runtime::block_on(engine.sync());
                    }
                }
                WindowEvent::Focused(focused) => {
                    if let Some(wf) = window.app_handle().try_state::<WindowFocus>() {
                        wf.set(*focused);
                    }
                    if !focused {
                        let app = window.app_handle().clone();
                        tauri::async_runtime::spawn(async move {
                            if let Some(engine) = app.try_state::<Arc<sync::SyncEngine>>() {
                                let _ = engine.push_pending().await;
                            }
                        });
                    }
                }
                _ => {}
            }
        })
        .setup(|app| {
            let handle = app.handle().clone();
            let sync_engine = app.state::<Arc<sync::SyncEngine>>().inner().clone();
            let window_focus = app.state::<WindowFocus>().inner().0.clone();

            // Periodic sync: every 5 min when focused and ready
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5 * 60));
                loop {
                    interval.tick().await;
                    let Some(engine) = handle.try_state::<Arc<sync::SyncEngine>>() else {
                        continue;
                    };
                    if !window_focus.load(Ordering::Relaxed) {
                        continue;
                    }
                    let Ok(status) = engine.status().await else { continue };
                    if !status.connected || status.needs_passphrase || engine.is_syncing() {
                        continue;
                    }
                    let _ = engine.sync().await;
                }
            });

            // WebSocket listener: connect when configured, sync on "sync" message
            tauri::async_runtime::spawn(sync::ws::run_ws_listener(sync_engine, app.handle().clone()));
            Ok(())
        })
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
            // Canvas commands
            canvas::get_canvases,
            canvas::get_canvas_by_id,
            canvas::create_canvas,
            canvas::update_canvas,
            canvas::delete_canvas,
            // Trash commands
            trash::get_trashed_tasks,
            trash::restore_task,
            // Activity commands
            activity::get_activities,
            // Search commands
            search::search_resources,
            // Link commands
            link::create_link,
            link::get_backlinks,
            link::get_outgoing_links,
            link::delete_link,
            link::search_linkable_resources,
            link::get_all_links_for_graph,
            link::sync_links_from_content,
            // Bookmark commands
            bookmark::get_bookmarks,
            bookmark::get_bookmark_by_id,
            bookmark::create_bookmark,
            bookmark::update_bookmark,
            bookmark::delete_bookmark,
            bookmark::add_tags_to_bookmark,
            bookmark::remove_tags_from_bookmark,
            bookmark::extract_metadata,
            // Audio commands
            commands::audio::save_audio_recording,
            commands::audio::get_audio_data,
            commands::audio::delete_audio_recording,
            commands::audio::get_media_items_for_entry,
            commands::audio::get_audio_metadata,
            // Transcription commands
            commands::transcription::start_transcription,
            commands::transcription::get_transcriptions,
            commands::transcription::get_transcription_by_id,
            commands::transcription::set_active_transcription,
            commands::transcription::list_providers,
            commands::transcription::validate_provider,
            commands::transcription::list_available_models,
            commands::transcription::download_model,
            commands::transcription::verify_model,
            commands::transcription::delete_model,
            // Settings commands
            commands::settings::get_setting,
            commands::settings::get_all_settings,
            commands::settings::set_setting,
            // Embedding model commands
            commands::embeddings::list_embedding_models,
            commands::embeddings::download_embedding_model,
            commands::embeddings::verify_embedding_model,
            commands::embeddings::delete_embedding_model,
            // Sync commands
            sync_commands::configure_sync,
            sync_commands::sync_now,
            sync_commands::get_sync_status,
            sync_commands::disconnect_sync,
            sync_commands::reconnect_sync,
            sync_commands::ensure_media_blob,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
