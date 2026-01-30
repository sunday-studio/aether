use crate::db::{connection, DbState, TaskRepository};
use crate::error::{AppError, Result};
use crate::handlers::common::PaginationResponse;
use crate::utils::{
    log_complete, log_create, log_delete, log_goal_operation, log_reorder, log_tag_operation,
    log_update,
};
use axum::{
    extract::{Path, Query, State},
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
        (status = 200, description = "Created task with subtasks", body = TaskWithSubtasks),
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

    let db = connection::get_database(&state);
    let repo = TaskRepository::new(db.clone());
    
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

    // Log activity
    if let Err(e) = log_create(db, "task".to_string(), task.id.clone()).await {
        tracing::warn!("Failed to log task creation activity: {}", e);
    }

    // New task has no subtasks yet
    Ok((StatusCode::OK, Json(TaskRepository::task_to_task_with_subtasks(task, vec![]))))
}

/// Get inbox tasks
#[utoipa::path(
    get,
    path = "/v1/tasks/inbox",
    tag = "Tasks",
    params(
        ("limit" = Option<u32>, Query, description = "Number of tasks per page (max 1000)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination")
    ),
    responses(
        (status = 200, description = "Paginated list of inbox tasks with subtasks", body = PaginatedTasksWithSubtasks),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_inbox_tasks(
    State(state): State<DbState>,
    Query(params): Query<crate::commands::params::PaginationQueryParams>,
) -> Result<impl IntoResponse> {
    let repo = TaskRepository::new(connection::get_database(&state));
    let (tasks, next_cursor, has_more) = repo
        .find_inbox(params.normalize_limit(), params.cursor)
        .await?;
    let tasks_with_subtasks = repo.with_subtasks(tasks).await?;
    Ok((
        StatusCode::OK,
        Json(PaginationResponse::new(tasks_with_subtasks, next_cursor, has_more)),
    ))
}

/// Get overdue tasks
#[utoipa::path(
    get,
    path = "/v1/tasks/overdue",
    tag = "Tasks",
    params(
        ("limit" = Option<u32>, Query, description = "Number of tasks per page (max 1000)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination")
    ),
    responses(
        (status = 200, description = "Paginated list of overdue tasks with subtasks", body = PaginatedTasksWithSubtasks),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_overdue_tasks(
    State(state): State<DbState>,
    Query(params): Query<crate::commands::params::PaginationQueryParams>,
) -> Result<impl IntoResponse> {
    let repo = TaskRepository::new(connection::get_database(&state));
    let (tasks, next_cursor, has_more) = repo
        .find_overdue(params.normalize_limit(), params.cursor)
        .await?;
    let tasks_with_subtasks = repo.with_subtasks(tasks).await?;
    Ok((
        StatusCode::OK,
        Json(PaginationResponse::new(tasks_with_subtasks, next_cursor, has_more)),
    ))
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
        (status = 200, description = "Task found with subtasks", body = TaskWithSubtasks),
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
        Some(task) => {
            let subtasks = repo.find_subtasks(&id).await?;
            Ok((StatusCode::OK, Json(TaskRepository::task_to_task_with_subtasks(task, subtasks))))
        }
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
        (status = 200, description = "Updated task with subtasks", body = TaskWithSubtasks),
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
    let db = connection::get_database(&state);
    let repo = TaskRepository::new(db.clone());
    
    // Get current task to check completion status
    let old_task = repo.find_by_id(&id).await?;
    let was_completed = old_task.as_ref().map(|t| t.is_completed).unwrap_or(false);
    
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

    // Log activity - check if completion changed
    if let Some(new_completed) = payload.is_completed {
        if !was_completed && new_completed {
            // Task was just completed
            if let Err(e) = log_complete(db.clone(), "task".to_string(), task.id.clone()).await {
                tracing::warn!("Failed to log task completion activity: {}", e);
            }
        } else {
            // Regular update
            if let Err(e) = log_update(db.clone(), "task".to_string(), task.id.clone()).await {
                tracing::warn!("Failed to log task update activity: {}", e);
            }
        }
    } else {
        // Regular update (no completion change)
        if let Err(e) = log_update(db.clone(), "task".to_string(), task.id.clone()).await {
            tracing::warn!("Failed to log task update activity: {}", e);
        }
    }

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

    // Get subtasks for the updated task
    let subtasks = repo.find_subtasks(&id).await?;
    Ok((StatusCode::OK, Json(TaskRepository::task_to_task_with_subtasks(task, subtasks))))
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
    let db = connection::get_database(&state);
    let repo = TaskRepository::new(db.clone());
    repo.delete(&id).await?;
    
    // Log activity
    if let Err(e) = log_delete(db, "task".to_string(), id.clone()).await {
        tracing::warn!("Failed to log task deletion activity for task {}: {}", id, e);
    }
    
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
        (status = 200, description = "List of subtasks", body = Vec<SubTask>),
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
        (status = 200, description = "Created subtask", body = SubTask),
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

    let db = connection::get_database(&state);
    let repo = TaskRepository::new(db.clone());
    let subtask = repo.create_subtask(&task_id, payload.title).await?;
    
    // Log activity
    if let Err(e) = log_create(db, "subtask".to_string(), subtask.id.clone()).await {
        tracing::warn!("Failed to log subtask creation activity: {}", e);
    }
    
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
        (status = 200, description = "Updated subtask", body = SubTask),
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
    let db = connection::get_database(&state);
    let repo = TaskRepository::new(db.clone());
    
    // Get current subtask to check completion status
    let subtasks = repo.find_subtasks(&task_id).await?;
    let old_subtask = subtasks.iter().find(|s| s.id == subtask_id);
    let was_completed = old_subtask.map(|s| s.is_completed).unwrap_or(false);
    
    let subtask = repo
        .update_subtask(&task_id, &subtask_id, payload.title, payload.is_completed)
        .await?;
    
    // Log activity - check if completion changed
    if let Some(new_completed) = payload.is_completed {
        if !was_completed && new_completed {
            // Subtask was just completed
            if let Err(e) = log_complete(db.clone(), "subtask".to_string(), subtask.id.clone()).await {
                tracing::warn!("Failed to log subtask completion activity: {}", e);
            }
        } else {
            // Regular update
            if let Err(e) = log_update(db.clone(), "subtask".to_string(), subtask.id.clone()).await {
                tracing::warn!("Failed to log subtask update activity: {}", e);
            }
        }
    } else {
        // Regular update (no completion change)
        if let Err(e) = log_update(db.clone(), "subtask".to_string(), subtask.id.clone()).await {
            tracing::warn!("Failed to log subtask update activity: {}", e);
        }
    }
    
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
    let db = connection::get_database(&state);
    let repo = TaskRepository::new(db.clone());
    repo.delete_subtask(&task_id, &subtask_id).await?;
    
    // Log activity
    if let Err(e) = log_delete(db, "subtask".to_string(), subtask_id.clone()).await {
        tracing::warn!("Failed to log subtask deletion activity for subtask {}: {}", subtask_id, e);
    }
    
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
    let db = connection::get_database(&state);
    let repo = TaskRepository::new(db.clone());
    repo.reorder_subtasks(&task_id, payload.sub_task_ids).await?;
    
    // Log activity for each reordered subtask (using task_id as entity_id since reorder affects the task's subtasks)
    // Actually, reorder is an action on subtasks, so we log it for the first subtask or the task itself
    // For simplicity, we'll log it once for the task
    if let Err(e) = log_reorder(db, "task".to_string(), task_id.clone()).await {
        tracing::warn!("Failed to log subtask reorder activity for task {}: {}", task_id, e);
    }
    
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
        (status = 200, description = "Tags added to task", body = TaskWithSubtasks),
        (status = 404, description = "Task or tag not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn add_tags_to_task(
    State(state): State<DbState>,
    Path(id): Path<String>,
    Json(tag_ids): Json<Vec<String>>,
) -> Result<impl IntoResponse> {
    let db = connection::get_database(&state);
    let repo = TaskRepository::new(db.clone());
    repo.add_tags(&id, tag_ids).await?;
    
    // Log activity
    if let Err(e) = log_tag_operation(db.clone(), "add_tags", "task".to_string(), id.clone()).await {
        tracing::warn!("Failed to log add_tags activity for task {}: {}", id, e);
    }
    
    // Return updated task with subtasks
    let task = repo.find_by_id(&id).await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", id)))?;
    let subtasks = repo.find_subtasks(&id).await?;
    Ok((StatusCode::OK, Json(TaskRepository::task_to_task_with_subtasks(task, subtasks))))
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
        (status = 200, description = "Tags removed from task", body = TaskWithSubtasks),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn remove_tags_from_task(
    State(state): State<DbState>,
    Path(id): Path<String>,
    Json(tag_ids): Json<Vec<String>>,
) -> Result<impl IntoResponse> {
    let db = connection::get_database(&state);
    let repo = TaskRepository::new(db.clone());
    repo.remove_tags(&id, tag_ids).await?;
    
    // Log activity
    if let Err(e) = log_tag_operation(db.clone(), "remove_tags", "task".to_string(), id.clone()).await {
        tracing::warn!("Failed to log remove_tags activity for task {}: {}", id, e);
    }
    
    // Return updated task with subtasks
    let task = repo.find_by_id(&id).await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", id)))?;
    let subtasks = repo.find_subtasks(&id).await?;
    Ok((StatusCode::OK, Json(TaskRepository::task_to_task_with_subtasks(task, subtasks))))
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
        (status = 200, description = "Goal added to task", body = TaskWithSubtasks),
        (status = 404, description = "Task or goal not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn add_goal_to_task(
    State(state): State<DbState>,
    Path(id): Path<String>,
    Json(payload): Json<AddGoalToTaskRequest>,
) -> Result<impl IntoResponse> {
    let db = connection::get_database(&state);
    let repo = TaskRepository::new(db.clone());
    
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
    
    // Log activity
    if let Err(e) = log_goal_operation(db, "add_goal", "task".to_string(), task.id.clone()).await {
        tracing::warn!("Failed to log add_goal activity for task {}: {}", task.id, e);
    }
    
    // Get subtasks for the updated task
    let subtasks = repo.find_subtasks(&id).await?;
    Ok((StatusCode::OK, Json(TaskRepository::task_to_task_with_subtasks(task, subtasks))))
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
        (status = 200, description = "Goal removed from task", body = TaskWithSubtasks),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn remove_goal_from_task(
    State(state): State<DbState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let db = connection::get_database(&state);
    let repo = TaskRepository::new(db.clone());
    repo.remove_goal(&id).await?;
    
    // Log activity
    if let Err(e) = log_goal_operation(db.clone(), "remove_goal", "task".to_string(), id.clone()).await {
        tracing::warn!("Failed to log remove_goal activity for task {}: {}", id, e);
    }
    
    // Return updated task with subtasks
    let task = repo.find_by_id(&id).await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", id)))?;
    let subtasks = repo.find_subtasks(&id).await?;
    Ok((StatusCode::OK, Json(TaskRepository::task_to_task_with_subtasks(task, subtasks))))
}
