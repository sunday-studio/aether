use utoipa::OpenApi;

use crate::commands::activity as activity_commands;
use crate::commands::common::{
    PaginatedEntries, PaginatedGoalInstances, PaginatedGoalInstancesWithTasks, PaginatedGoals,
    PaginatedTags, PaginatedTasks, PaginatedTasksWithSubtasks,
};
use crate::commands::embeddings as embedding_commands;
use crate::commands::entry as entry_commands;
use crate::commands::goal as goal_commands;
use crate::commands::media as media_commands;
use crate::commands::search as search_commands;
use crate::commands::settings as settings_commands;
use crate::commands::sync as sync_commands;
use crate::commands::tag as tag_commands;
use crate::commands::task as task_commands;
use crate::commands::trash as trash_commands;
use crate::db::models::{
    Activity, Entry, Goal, GoalInstance, GoalInstanceWithTasks, MediaItem, SubTask, Tag, Task,
    TaskWithSubtasks,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        // Tag endpoints
        tag_commands::get_all_tags,
        tag_commands::create_tag,
        tag_commands::bulk_create_tags,
        // Entry endpoints
        entry_commands::get_entries,
        entry_commands::get_entry_by_id,
        entry_commands::create_entry,
        entry_commands::bulk_create_entries,
        entry_commands::update_entry,
        entry_commands::delete_entry,
        entry_commands::add_tags_to_entry,
        entry_commands::remove_tags_from_entry,
        // Task endpoints
        task_commands::create_task,
        task_commands::get_inbox_tasks,
        task_commands::get_overdue_tasks,
        task_commands::get_task_by_id,
        task_commands::update_task,
        task_commands::delete_task,
        task_commands::get_subtasks,
        task_commands::create_subtask,
        task_commands::update_subtask,
        task_commands::delete_subtask,
        task_commands::reorder_subtasks,
        task_commands::add_tags_to_task,
        task_commands::remove_tags_from_task,
        task_commands::add_goal_to_task,
        task_commands::remove_goal_from_task,
        // Goal endpoints
        goal_commands::get_goals,
        goal_commands::get_goal_by_id,
        goal_commands::create_goal,
        goal_commands::update_goal,
        goal_commands::delete_goal,
        goal_commands::get_goal_instances,
        goal_commands::get_current_goal_instance,
        goal_commands::add_tags_to_goal,
        goal_commands::remove_tags_from_goal,
        // Trash endpoints
        trash_commands::get_trashed_tasks,
        trash_commands::restore_task,
        // Activity endpoints
        activity_commands::get_activities,
        // Search endpoints
        search_commands::search_resources,
        search_commands::reindex_search,
        search_commands::reindex_search_resource,
        search_commands::get_search_index_status,
        embedding_commands::index_search_embeddings,
        embedding_commands::index_search_resource_embeddings,
        embedding_commands::get_search_embedding_status,
        // Settings endpoints
        settings_commands::get_setting,
        settings_commands::get_all_settings,
        settings_commands::set_setting,
        // Media endpoints
        media_commands::get_media_items_for_entry,
        // Sync endpoints
        sync_commands::configure_sync,
        sync_commands::sync_now,
        sync_commands::get_sync_status,
        sync_commands::disconnect_sync,
        sync_commands::reconnect_sync,
        sync_commands::ensure_media_blob,
    ),
    components(schemas(
        // Base models
        Tag,
        Entry,
        Task,
        SubTask,
        Goal,
        GoalInstance,
        GoalInstanceWithTasks,
        Activity,
        MediaItem,
        // Pagination response types (concrete aliases for proper OpenAPI generation)
        PaginatedEntries,
        PaginatedTags,
        PaginatedTasks,
        PaginatedTasksWithSubtasks,
        PaginatedGoals,
        PaginatedGoalInstances,
        PaginatedGoalInstancesWithTasks,
        // Composite types
        TaskWithSubtasks,
        // Request/Response schemas
        tag_commands::CreateTagRequest,
        entry_commands::CreateEntryRequest,
        entry_commands::UpdateEntryRequest,
        task_commands::CreateTaskRequest,
        task_commands::UpdateTaskRequest,
        task_commands::CreateSubTaskRequest,
        task_commands::UpdateSubTaskRequest,
        task_commands::ReorderSubTasksRequest,
        task_commands::AddGoalToTaskRequest,
        goal_commands::CreateGoalRequest,
        goal_commands::UpdateGoalRequest,
        search_commands::SearchRequest,
        search_commands::SearchResultResponse,
        search_commands::SearchResponse,
        search_commands::ReindexResourceRequest,
        crate::db::repositories::SearchIndexStatus,
        embedding_commands::IndexSearchEmbeddingsRequest,
        embedding_commands::IndexSearchResourceEmbeddingsRequest,
        crate::db::repositories::SearchEmbeddingModelStatus,
        crate::db::repositories::SearchEmbeddingStatus,
        settings_commands::SettingResponse,
        settings_commands::AllSettingsResponse,
        settings_commands::SetSettingRequest,
        crate::sync::SyncStatus,
        sync_commands::ConfigureSyncRequest,
        sync_commands::ReconnectSyncRequest,
    )),
    tags(
        (name = "Tags", description = "Tag management endpoints"),
        (name = "Entries", description = "Entry management endpoints"),
        (name = "Tasks", description = "Task management endpoints"),
        (name = "Goals", description = "Goal management endpoints"),
        (name = "GoalInstances", description = "Goal instance management endpoints"),
        (name = "Trash", description = "Trash management endpoints"),
        (name = "Activities", description = "Activity tracking endpoints"),
        (name = "Search", description = "Search endpoints"),
        (name = "Settings", description = "Settings management endpoints"),
        (name = "Media", description = "Image and video media endpoints"),
        (name = "Sync", description = "Sync management endpoints"),
    ),
)]
pub struct ApiDoc;

/// Get the OpenAPI spec as JSON string
/// This is used for build-time spec generation
pub fn get_openapi_json() -> String {
    let openapi = ApiDoc::openapi();
    serde_json::to_string_pretty(&openapi).unwrap_or_else(|e| {
        panic!("Failed to serialize OpenAPI spec: {}", e);
    })
}

// pub fn swagger_ui() -> SwaggerUi {
//     SwaggerUi::new("/swagger/{*path}").url("/api-doc/openapi.json", ApiDoc::openapi())
// }
