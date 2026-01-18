use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::db::models::{Entry, Goal, GoalInstance, SubTask, Tag, Task};
use crate::handlers::entry;
use crate::handlers::goal as goal_handlers;
use crate::handlers::sync as sync_handlers;
use crate::handlers::tag;
use crate::handlers::task as task_handlers;
use crate::handlers::trash as trash_handlers;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Tag endpoints
        tag::get_all_tags,
        tag::create_tag,
        tag::bulk_create_tags,
        // Entry endpoints
        entry::get_entries,
        entry::get_entry_by_id,
        entry::create_entry,
        entry::bulk_create_entries,
        entry::update_entry,
        entry::delete_entry,
        entry::add_tags_to_entry,
        entry::remove_tags_from_entry,
        // Task endpoints
        task_handlers::create_task,
        task_handlers::get_inbox_tasks,
        task_handlers::get_overdue_tasks,
        task_handlers::get_task_by_id,
        task_handlers::update_task,
        task_handlers::delete_task,
        task_handlers::get_subtasks,
        task_handlers::create_subtask,
        task_handlers::update_subtask,
        task_handlers::delete_subtask,
        task_handlers::reorder_subtasks,
        task_handlers::add_tags_to_task,
        task_handlers::remove_tags_from_task,
        task_handlers::add_goal_to_task,
        task_handlers::remove_goal_from_task,
        // Goal endpoints
        goal_handlers::get_goals,
        goal_handlers::get_goal_by_id,
        goal_handlers::create_goal,
        goal_handlers::update_goal,
        goal_handlers::delete_goal,
        goal_handlers::get_goal_instances,
        goal_handlers::get_current_goal_instance,
        goal_handlers::add_tags_to_goal,
        goal_handlers::remove_tags_from_goal,
        // Trash endpoints
        trash_handlers::get_trashed_tasks,
        trash_handlers::restore_task,
        // Sync endpoints
        sync_handlers::configure_sync,
        sync_handlers::sync,
    ),
    components(schemas(
        Tag,
        Entry,
        Task,
        SubTask,
        Goal,
        GoalInstance,
        tag::CreateTagRequest,
        entry::CreateEntryRequest,
        entry::UpdateEntryRequest,
        task_handlers::CreateTaskRequest,
        task_handlers::UpdateTaskRequest,
        task_handlers::CreateSubTaskRequest,
        task_handlers::UpdateSubTaskRequest,
        task_handlers::ReorderSubTasksRequest,
        task_handlers::AddGoalToTaskRequest,
        goal_handlers::CreateGoalRequest,
        goal_handlers::UpdateGoalRequest,
        sync_handlers::ConfigureSyncRequest,
    )),
    tags(
        (name = "Tags", description = "Tag management endpoints"),
        (name = "Entries", description = "Entry management endpoints"),
        (name = "Tasks", description = "Task management endpoints"),
        (name = "Goals", description = "Goal management endpoints"),
        (name = "GoalInstances", description = "Goal instance management endpoints"),
        (name = "Trash", description = "Trash management endpoints"),
        (name = "Sync", description = "Sync management endpoints"),
    ),
)]
pub struct ApiDoc;

// pub fn swagger_ui() -> SwaggerUi {
//     SwaggerUi::new("/swagger/{*path}").url("/api-doc/openapi.json", ApiDoc::openapi())
// }
