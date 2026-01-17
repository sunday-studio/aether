use crate::db::DbState;
use crate::handlers::{entry, goal, sync, tag, task, trash};
use axum::{
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde_json::json;

/// Register all API routes
pub fn register_routes(state: DbState) -> Router {
    Router::new()
        .route("/v1/ping", get(ping))
        // Tag routes
        .route("/v1/tags", get(tag::get_all_tags).post(tag::create_tag))
        .route("/v1/tags/bulk-create", post(tag::bulk_create_tags))
        // Entry routes
        .route("/v1/entry", get(entry::get_entries).post(entry::create_entry))
        .route("/v1/entry/bulk-create", post(entry::bulk_create_entries))
        .route(
            "/v1/entry/:id",
            get(entry::get_entry_by_id)
                .put(entry::update_entry)
                .delete(entry::delete_entry),
        )
        .route("/v1/entry/:id/tags", post(entry::add_tags_to_entry).delete(entry::remove_tags_from_entry))
        // Task routes
        .route("/v1/tasks", post(task::create_task))
        .route("/v1/tasks/inbox", get(task::get_inbox_tasks))
        .route("/v1/tasks/overdue", get(task::get_overdue_tasks))
        .route(
            "/v1/tasks/:id",
            get(task::get_task_by_id)
                .put(task::update_task)
                .delete(task::delete_task),
        )
        .route("/v1/tasks/:id/tags", post(task::add_tags_to_task).delete(task::remove_tags_from_task))
        .route("/v1/tasks/:id/goal", post(task::add_goal_to_task).delete(task::remove_goal_from_task))
        .route("/v1/tasks/:taskId/subtasks", get(task::get_subtasks).post(task::create_subtask))
        .route(
            "/v1/tasks/:taskId/subtasks/:subtaskId",
            put(task::update_subtask).delete(task::delete_subtask),
        )
        .route("/v1/tasks/:taskId/subtasks/reorder", post(task::reorder_subtasks))
        // Goal routes
        .route("/v1/goals", get(goal::get_goals).post(goal::create_goal))
        .route(
            "/v1/goals/:id",
            get(goal::get_goal_by_id)
                .put(goal::update_goal)
                .delete(goal::delete_goal),
        )
        .route("/v1/goals/:id/tags", post(goal::add_tags_to_goal).delete(goal::remove_tags_from_goal))
        .route("/v1/goals/:goalId/instances", get(goal::get_goal_instances))
        .route("/v1/goals/:goalId/instances/current", get(goal::get_current_goal_instance))
        // Trash routes
        .route("/v1/trash/tasks", get(trash::get_trashed_tasks))
        .route("/v1/trash/:id/restore", post(trash::restore_task))
        // Sync route
        .route("/v1/sync", post(sync::sync))
        .with_state(state)
}

/// Health check endpoint
/// GET /v1/ping
#[utoipa::path(
    get,
    path = "/v1/ping",
    responses(
        (status = 200, description = "Health check response")
    )
)]
async fn ping() -> Json<serde_json::Value> {
    Json(json!({
        "message": "pong pong"
    }))
}
