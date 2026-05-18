pub mod api;
pub mod audio;
pub mod commands;
pub mod db;
pub mod error;
pub mod media;
pub mod settings;
pub mod sync;
pub mod transcription;
pub mod updater;
pub mod utils;

pub use db::DbState;
pub use error::{AppError, Result};

use commands::{
    activity, ai_journal, bookmark, canvas, entry, goal, link, search, sync as sync_commands, tag,
    task, trash, updater as updater_commands,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, WindowEvent};
use tokio::sync::oneshot;

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

/// One-shot sender to start the periodic sync task when user configures sync (no URL on first load).
pub struct StartPeriodicSyncTx(pub Arc<Mutex<Option<oneshot::Sender<()>>>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // Default: info level globally, debug for our crate (includes activity logging)
                // You can override with RUST_LOG env var, e.g.:
                // RUST_LOG=debug pnpm tauri dev
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

    builder = builder.plugin(tauri_plugin_keyring::init());
    builder = builder.plugin(tauri_plugin_updater::Builder::new().build());

    // Initialize database and run migrations
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let db_state = rt.block_on(async {
        // Load environment variables
        dotenvy::dotenv().ok();

        // Initialize database
        let state = db::initialize(None)
            .await
            .expect("Failed to initialize database");

        // Run migrations
        let database = db::connection::get_database(&state);
        db::migrations::run_migrations(&database)
            .await
            .expect("Failed to run migrations");

        // Ensure media directory exists
        media::ensure_media_directory().expect("Failed to create media directory");

        state
    });

    let sync_engine = Arc::new(sync::SyncEngine::new(db_state.clone()));

    // Ensure _suppress_triggers is reset to '0' on startup (in case it was stuck)
    let db = db::connection::get_database(&db_state);
    let _ = rt.block_on(async {
        use crate::sync::metadata;
        match metadata::get_suppress_triggers(db.as_ref()).await {
            Ok(suppress) => {
                if suppress != "0" {
                    tracing::warn!(
                        "[SYNC] _suppress_triggers was '{}' on startup, resetting to '0'",
                        suppress
                    );
                    let _ = metadata::set_suppress_triggers(db.as_ref(), "0").await;
                } else {
                    tracing::debug!("[SYNC] _suppress_triggers is correctly set to '0'");
                }
            }
            Err(e) => {
                tracing::warn!(
                    "[SYNC] Could not check _suppress_triggers on startup: {}",
                    e
                );
            }
        }
    });

    let window_focus = WindowFocus(Arc::new(AtomicBool::new(true)));
    let update_manager = updater::UpdateManager::new();

    let (periodic_sync_tx, periodic_sync_rx) = oneshot::channel();
    builder
        .manage(db_state)
        .manage(sync_engine.clone())
        .manage(window_focus)
        .manage(update_manager)
        .manage(StartPeriodicSyncTx(Arc::new(Mutex::new(Some(periodic_sync_tx)))))
        .on_window_event(|window, event| {
            match event {
                WindowEvent::CloseRequested { .. } => {
                    tracing::info!("[SYNC] Window closing, running final sync");
                    if let Some(engine) = window.app_handle().try_state::<Arc<sync::SyncEngine>>() {
                        match tauri::async_runtime::block_on(engine.sync()) {
                            Ok(status) => {
                                tracing::info!("[SYNC] Final sync completed: pending={}, last_sync={:?}",
                                    status.pending_changes, status.last_sync);
                            }
                            Err(e) => {
                                tracing::error!("[SYNC] Final sync failed: {}", e);
                            }
                        }
                    }
                }
                WindowEvent::Focused(focused) => {
                    if let Some(wf) = window.app_handle().try_state::<WindowFocus>() {
                        wf.set(*focused);
                    }
                    if !focused {
                        tracing::info!("[SYNC] Window lost focus, pushing pending changes");
                        let app = window.app_handle().clone();
                        tauri::async_runtime::spawn(async move {
                            if let Some(engine) = app.try_state::<Arc<sync::SyncEngine>>() {
                                match engine.push_pending().await {
                                    Ok(()) => tracing::debug!("[SYNC] Pushed pending changes on window blur"),
                                    Err(e) => tracing::warn!("[SYNC] Failed to push pending changes on window blur: {}", e),
                                }
                            }
                        });
                    } else {
                        tracing::info!("[SYNC] Window gained focus, triggering sync");
                        let app = window.app_handle().clone();
                        tauri::async_runtime::spawn(async move {
                            if let Some(engine) = app.try_state::<Arc<sync::SyncEngine>>() {
                                if let Ok(status) = engine.status().await {
                                    if status.connected && !status.needs_passphrase && !engine.is_syncing() {
                                        match engine.sync().await {
                                            Ok(new_status) => {
                                                tracing::info!(
                                                    "[SYNC] Focus sync completed: pending={}, last_sync={:?}",
                                                    new_status.pending_changes,
                                                    new_status.last_sync
                                                );
                                                let _ = app.emit("sync-status", &new_status);
                                            }
                                            Err(e) => tracing::warn!("[SYNC] Focus sync failed: {}", e),
                                        }
                                    }
                                }
                            }
                            if let Some(manager) = app.try_state::<updater::UpdateManager>() {
                                if manager.should_check().await {
                                    tracing::debug!("[UPDATER] Checking for updates on focus");
                                    let result = updater::check_for_updates(&app).await;
                                    let failed = result.is_err();
                                    match result {
                                        Ok(Some(info)) => {
                                            if !manager.is_version_skipped(&info.latest_version).await {
                                                tracing::info!("[UPDATER] Update available: v{}", info.latest_version);
                                                let _ = app.emit("update-available", &info);
                                            }
                                        }
                                        Ok(None) => tracing::debug!("[UPDATER] No update available"),
                                        Err(e) => tracing::warn!("[UPDATER] Failed to check for updates (will back off): {}", e),
                                    }
                                    manager.record_check(failed).await;
                                }
                            }
                        });
                    }
                }
                _ => {}
            }
        })
        .setup(move |app| {
            let handle = app.handle().clone();
            let sync_engine = app.state::<Arc<sync::SyncEngine>>().inner().clone();
            let window_focus = app.state::<WindowFocus>().inner().0.clone();
            let app_handle = app.handle().clone();
            let engine_clone = sync_engine.clone();
            let update_manager = app.state::<updater::UpdateManager>().inner().clone();
            let updater_app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                update_manager.hydrate(&updater_app_handle).await;
            });

            tauri::async_runtime::spawn(async move {
                if let Err(e) = engine_clone.hydrate(&app_handle).await {
                    tracing::warn!("[SYNC] Failed to hydrate sync engine: {}", e);
                }
                let has_url = match handle.try_state::<Arc<sync::SyncEngine>>() {
                    Some(engine) => engine.status().await.ok().map(|s| s.connected).unwrap_or(false),
                    None => false,
                };
                if !has_url {
                    tracing::info!("[SYNC] No sync server URL configured, waiting for user to set sync server before starting periodic sync");
                    let _ = periodic_sync_rx.await;
                }
                tracing::info!("[SYNC] Starting periodic sync task (every 5 minutes)");
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
                loop {
                    interval.tick().await;
                    let Some(engine) = handle.try_state::<Arc<sync::SyncEngine>>() else {
                        tracing::warn!("[SYNC] Periodic sync: engine not available");
                        continue;
                    };
                    if !window_focus.load(Ordering::Relaxed) {
                        tracing::debug!("[SYNC] Periodic sync: window not focused, skipping");
                        continue;
                    }
                    let Ok(status) = engine.status().await else {
                        tracing::warn!("[SYNC] Periodic sync: failed to get status");
                        continue;
                    };
                    if !status.connected {
                        tracing::debug!("[SYNC] Periodic sync: not connected, skipping");
                        continue;
                    }
                    if status.needs_passphrase {
                        tracing::debug!("[SYNC] Periodic sync: needs passphrase, skipping");
                        continue;
                    }
                    if engine.is_syncing() {
                        tracing::debug!("[SYNC] Periodic sync: sync already in progress, skipping");
                        continue;
                    }
                    tracing::info!("[SYNC] Periodic sync triggered");
                    match engine.sync().await {
                        Ok(status) => {
                            tracing::info!("[SYNC] Periodic sync completed: pending={}, last_sync={:?}",
                                status.pending_changes, status.last_sync);
                        }
                        Err(e) => {
                            tracing::error!("[SYNC] Periodic sync failed: {}", e);
                        }
                    }
                }
            });

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
            search::reindex_search,
            search::reindex_search_resource,
            search::get_search_index_status,
            search::find_related_resources,
            search::retrieve_context,
            search::retrieve_week_context,
            // AI journal commands
            ai_journal::enrich_journal_entry,
            ai_journal::get_entry_insights,
            ai_journal::update_ai_suggestion,
            ai_journal::accept_ai_tag_suggestion,
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
            commands::embeddings::index_search_embeddings,
            commands::embeddings::index_search_resource_embeddings,
            commands::embeddings::get_search_embedding_status,
            // Sync commands
            sync_commands::configure_sync,
            sync_commands::sync_now,
            sync_commands::get_sync_status,
            sync_commands::disconnect_sync,
            sync_commands::reconnect_sync,
            sync_commands::ensure_media_blob,
            sync_commands::check_sync_triggers,
            sync_commands::test_sync_trigger,
            // Updater commands
            updater_commands::check_for_updates,
            updater_commands::download_and_install_update,
            updater_commands::skip_update_version,
            updater_commands::get_update_preferences,
            updater_commands::set_update_preferences,
            updater_commands::get_app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
