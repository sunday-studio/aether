use crate::db::{connection, DbState, TaskRepository};
use crate::error::{AppError, Result};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub due_date: Option<chrono::DateTime<Utc>>,
    #[serde(default)]
    pub goal_id: Option<String>,
    #[serde(default)]
    pub tag_ids: Vec<String>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<Option<String>>,
    #[serde(default)]
    pub due_date: Option<Option<chrono::DateTime<Utc>>>,
    #[serde(default)]
    pub is_completed: Option<bool>,
    #[serde(default)]
    pub goal_id: Option<Option<String>>,
    #[serde(default)]
    pub tag_ids: Option<Vec<String>>,
    #[serde(default)]
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubTaskRequest {
    pub title: String,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubTaskRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub is_completed: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReorderSubTasksRequest {
    pub sub_task_ids: Vec<String>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddGoalToTaskRequest {
    pub goal_id: String,
}

/// Create a new task
#[utoipa::path(
    post,
    path = "/v1/tasks",
    tag = "Tasks",
    request_body = CreateTaskRequest,
    responses(
        (status = 200, description = "Created task", body = crate::db::models::Task),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_task(
    State(state): State<DbState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<impl IntoResponse> {
    if payload.title.is_empty() {
        return Err(AppError::BadRequest("Title is required".to_string()));
    }

    let repo = TaskRepository::new(connection::get_database(&state));
    
    // For now, goal_instance_id will be None - will be implemented in Milestone 5
    let task = repo
        .create(
            payload.title,
            payload.description,
            payload.due_date,
            payload.goal_id.clone(),
            None, // goal_instance_id - will be set in Milestone 5
        )
        .await?;

    // Add tags if provided
    if !payload.tag_ids.is_empty() {
        repo.add_tags(&task.id, payload.tag_ids).await?;
    }

    Ok((StatusCode::OK, Json(task)))
}

/// Get inbox tasks
#[utoipa::path(
    get,
    path = "/v1/tasks/inbox",
    tag = "Tasks",
    responses(
        (status = 200, description = "List of inbox tasks", body = Vec<crate::db::models::Task>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_inbox_tasks(State(state): State<DbState>) -> Result<impl IntoResponse> {
    let repo = TaskRepository::new(connection::get_database(&state));
    let tasks = repo.find_inbox().await?;
    Ok((StatusCode::OK, Json(tasks)))
}

/// Get overdue tasks
#[utoipa::path(
    get,
    path = "/v1/tasks/overdue",
    tag = "Tasks",
    responses(
        (status = 200, description = "List of overdue tasks", body = Vec<crate::db::models::Task>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_overdue_tasks(State(state): State<DbState>) -> Result<impl IntoResponse> {
    let repo = TaskRepository::new(connection::get_database(&state));
    let tasks = repo.find_overdue().await?;
    Ok((StatusCode::OK, Json(tasks)))
}

/// Get task by ID
#[utoipa::path(
    get,
    path = "/v1/tasks/{id}",
    tag = "Tasks",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task found", body = crate::db::models::Task),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_task_by_id(
    State(state): State<DbState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let repo = TaskRepository::new(connection::get_database(&state));
    match repo.find_by_id(&id).await? {
        Some(task) => Ok((StatusCode::OK, Json(task))),
        None => Err(AppError::NotFound(format!("Task {} not found", id))),
    }
}

/// Update a task
#[utoipa::path(
    put,
    path = "/v1/tasks/{id}",
    tag = "Tasks",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    request_body = UpdateTaskRequest,
    responses(
        (status = 200, description = "Updated task", body = crate::db::models::Task),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Task not found"),
        (status = 409, description = "Conflict: Task was modified by another device"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_task(
    State(state): State<DbState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<impl IntoResponse> {
    let repo = TaskRepository::new(connection::get_database(&state));
    
    // For goal_id, we'll need to get or create goal instance in Milestone 5
    // For now, pass None for goal_instance_id
    let task = repo
        .update(
            &id,
            payload.title,
            payload.description,
            payload.due_date,
            payload.is_completed,
            payload.goal_id,
            None, // goal_instance_id - will be set in Milestone 5
            payload.updated_at,
        )
        .await?;

    // Update tags if provided
    if let Some(tag_ids) = payload.tag_ids {
        // For simplicity, we'll replace all tags
        // First remove all existing tags, then add new ones
        // This could be optimized in the future
        let current_task = repo.find_by_id(&id).await?;
        if let Some(_task) = current_task {
            // Get current tags and remove them, then add new ones
            // For now, we'll just add the new tags (handlers will manage this better)
            repo.add_tags(&id, tag_ids).await?;
        }
    }

    Ok((StatusCode::OK, Json(task)))
}

/// Delete a task
#[utoipa::path(
    delete,
    path = "/v1/tasks/{id}",
    tag = "Tasks",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 204, description = "Task deleted"),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_task(
    State(state): State<DbState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let repo = TaskRepository::new(connection::get_database(&state));
    repo.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get subtasks for a task
#[utoipa::path(
    get,
    path = "/v1/tasks/{taskId}/subtasks",
    tag = "Tasks",
    params(
        ("taskId" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "List of subtasks", body = Vec<crate::db::models::SubTask>),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_subtasks(
    State(state): State<DbState>,
    Path(task_id): Path<String>,
) -> Result<impl IntoResponse> {
    let repo = TaskRepository::new(connection::get_database(&state));
    let subtasks = repo.find_subtasks(&task_id).await?;
    Ok((StatusCode::OK, Json(subtasks)))
}

/// Create a subtask
#[utoipa::path(
    post,
    path = "/v1/tasks/{taskId}/subtasks",
    tag = "Tasks",
    params(
        ("taskId" = String, Path, description = "Task ID")
    ),
    request_body = CreateSubTaskRequest,
    responses(
        (status = 200, description = "Created subtask", body = crate::db::models::SubTask),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_subtask(
    State(state): State<DbState>,
    Path(task_id): Path<String>,
    Json(payload): Json<CreateSubTaskRequest>,
) -> Result<impl IntoResponse> {
    if payload.title.is_empty() {
        return Err(AppError::BadRequest("Title is required".to_string()));
    }

    let repo = TaskRepository::new(connection::get_database(&state));
    let subtask = repo.create_subtask(&task_id, payload.title).await?;
    Ok((StatusCode::OK, Json(subtask)))
}

/// Update a subtask
#[utoipa::path(
    put,
    path = "/v1/tasks/{taskId}/subtasks/{subtaskId}",
    tag = "Tasks",
    params(
        ("taskId" = String, Path, description = "Task ID"),
        ("subtaskId" = String, Path, description = "Subtask ID")
    ),
    request_body = UpdateSubTaskRequest,
    responses(
        (status = 200, description = "Updated subtask", body = crate::db::models::SubTask),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Subtask not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_subtask(
    State(state): State<DbState>,
    Path((task_id, subtask_id)): Path<(String, String)>,
    Json(payload): Json<UpdateSubTaskRequest>,
) -> Result<impl IntoResponse> {
    let repo = TaskRepository::new(connection::get_database(&state));
    let subtask = repo
        .update_subtask(&task_id, &subtask_id, payload.title, payload.is_completed)
        .await?;
    Ok((StatusCode::OK, Json(subtask)))
}

/// Delete a subtask
#[utoipa::path(
    delete,
    path = "/v1/tasks/{taskId}/subtasks/{subtaskId}",
    tag = "Tasks",
    params(
        ("taskId" = String, Path, description = "Task ID"),
        ("subtaskId" = String, Path, description = "Subtask ID")
    ),
    responses(
        (status = 204, description = "Subtask deleted"),
        (status = 404, description = "Subtask not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_subtask(
    State(state): State<DbState>,
    Path((task_id, subtask_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    let repo = TaskRepository::new(connection::get_database(&state));
    repo.delete_subtask(&task_id, &subtask_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Reorder subtasks
#[utoipa::path(
    post,
    path = "/v1/tasks/{taskId}/subtasks/reorder",
    tag = "Tasks",
    params(
        ("taskId" = String, Path, description = "Task ID")
    ),
    request_body = ReorderSubTasksRequest,
    responses(
        (status = 200, description = "Subtasks reordered"),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn reorder_subtasks(
    State(state): State<DbState>,
    Path(task_id): Path<String>,
    Json(payload): Json<ReorderSubTasksRequest>,
) -> Result<impl IntoResponse> {
    let repo = TaskRepository::new(connection::get_database(&state));
    repo.reorder_subtasks(&task_id, payload.sub_task_ids).await?;
    Ok((StatusCode::OK, Json(serde_json::json!({"success": true}))))
}

/// Add tags to a task
#[utoipa::path(
    post,
    path = "/v1/tasks/{id}/tags",
    tag = "Tasks",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    request_body = Vec<String>,
    responses(
        (status = 200, description = "Tags added to task"),
        (status = 404, description = "Task or tag not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn add_tags_to_task(
    State(state): State<DbState>,
    Path(id): Path<String>,
    Json(tag_ids): Json<Vec<String>>,
) -> Result<impl IntoResponse> {
    let repo = TaskRepository::new(connection::get_database(&state));
    repo.add_tags(&id, tag_ids).await?;
    
    // Return updated task
    let task = repo.find_by_id(&id).await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", id)))?;
    Ok((StatusCode::OK, Json(task)))
}

/// Remove tags from a task
#[utoipa::path(
    delete,
    path = "/v1/tasks/{id}/tags",
    tag = "Tasks",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    request_body = Vec<String>,
    responses(
        (status = 200, description = "Tags removed from task"),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn remove_tags_from_task(
    State(state): State<DbState>,
    Path(id): Path<String>,
    Json(tag_ids): Json<Vec<String>>,
) -> Result<impl IntoResponse> {
    let repo = TaskRepository::new(connection::get_database(&state));
    repo.remove_tags(&id, tag_ids).await?;
    
    // Return updated task
    let task = repo.find_by_id(&id).await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", id)))?;
    Ok((StatusCode::OK, Json(task)))
}

/// Add goal to a task
#[utoipa::path(
    post,
    path = "/v1/tasks/{id}/goal",
    tag = "Tasks",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    request_body = AddGoalToTaskRequest,
    responses(
        (status = 200, description = "Goal added to task", body = crate::db::models::Task),
        (status = 404, description = "Task or goal not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn add_goal_to_task(
    State(state): State<DbState>,
    Path(id): Path<String>,
    Json(payload): Json<AddGoalToTaskRequest>,
) -> Result<impl IntoResponse> {
    let repo = TaskRepository::new(connection::get_database(&state));
    
    // Get or create goal instance - placeholder for now, will be implemented in Milestone 5
    let goal_instance_id = repo.add_goal(&id, &payload.goal_id).await?;
    
    // Update task with goal
    let task = repo
        .update(
            &id,
            None, // title
            None, // description
            None, // due_date
            None, // is_completed
            Some(Some(payload.goal_id.clone())),
            Some(goal_instance_id),
            None, // client_updated_at
        )
        .await?;
    
    Ok((StatusCode::OK, Json(task)))
}

/// Remove goal from a task
#[utoipa::path(
    delete,
    path = "/v1/tasks/{id}/goal",
    tag = "Tasks",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Goal removed from task", body = crate::db::models::Task),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn remove_goal_from_task(
    State(state): State<DbState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let repo = TaskRepository::new(connection::get_database(&state));
    repo.remove_goal(&id).await?;
    
    // Return updated task
    let task = repo.find_by_id(&id).await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", id)))?;
    Ok((StatusCode::OK, Json(task)))
}
