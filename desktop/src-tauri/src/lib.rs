
use aether_backend::commands::{
    entry, goal, sync, tag, task, trash,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
        let state = aether_backend::db::initialize().await
            .expect("Failed to initialize database");
        
        // Run migrations
        let database = aether_backend::db::connection::get_database(&state);
        aether_backend::db::migrations::run_migrations(&database).await
            .expect("Failed to run migrations");
        
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}