use crate::db::{connection, DbState, TaskRepository};
use crate::error::{AppError, Result};
use crate::handlers::task::{
    AddGoalToTaskRequest, CreateSubTaskRequest, CreateTaskRequest, ReorderSubTasksRequest,
    UpdateSubTaskRequest, UpdateTaskRequest,
};
use tauri::State;

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
#[tauri::command]
pub async fn create_task(
    state: State<'_, DbState>,
    payload: CreateTaskRequest,
) -> Result<crate::db::models::Task> {
    if payload.title.is_empty() {
        return Err(AppError::BadRequest("Title is required".to_string()));
    }

    let repo = TaskRepository::new(connection::get_database(&*state));
    
    let task = repo
        .create(
            payload.title,
            payload.description,
            payload.due_date,
            payload.goal_id.clone(),
            None, // goal_instance_id - will be set in Milestone 5
        )
        .await?;

    if !payload.tag_ids.is_empty() {
        repo.add_tags(&task.id, payload.tag_ids).await?;
    }

    Ok(task)
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
#[tauri::command]
pub async fn get_inbox_tasks(state: State<'_, DbState>) -> Result<Vec<crate::db::models::Task>> {
    let repo = TaskRepository::new(connection::get_database(&*state));
    repo.find_inbox().await
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
#[tauri::command]
pub async fn get_overdue_tasks(state: State<'_, DbState>) -> Result<Vec<crate::db::models::Task>> {
    let repo = TaskRepository::new(connection::get_database(&*state));
    repo.find_overdue().await
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
#[tauri::command]
pub async fn get_task_by_id(
    state: State<'_, DbState>,
    id: String,
) -> Result<crate::db::models::Task> {
    let repo = TaskRepository::new(connection::get_database(&*state));
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", id)))
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
#[tauri::command]
pub async fn update_task(
    state: State<'_, DbState>,
    id: String,
    payload: UpdateTaskRequest,
) -> Result<crate::db::models::Task> {
    let repo = TaskRepository::new(connection::get_database(&*state));
    
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

    if let Some(tag_ids) = payload.tag_ids {
        let current_task = repo.find_by_id(&id).await?;
        if current_task.is_some() {
            repo.add_tags(&id, tag_ids).await?;
        }
    }

    Ok(task)
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
#[tauri::command]
pub async fn delete_task(state: State<'_, DbState>, id: String) -> Result<()> {
    let repo = TaskRepository::new(connection::get_database(&*state));
    repo.delete(&id).await
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
#[tauri::command]
pub async fn get_subtasks(
    state: State<'_, DbState>,
    task_id: String,
) -> Result<Vec<crate::db::models::SubTask>> {
    let repo = TaskRepository::new(connection::get_database(&*state));
    repo.find_subtasks(&task_id).await
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
#[tauri::command]
pub async fn create_subtask(
    state: State<'_, DbState>,
    task_id: String,
    payload: CreateSubTaskRequest,
) -> Result<crate::db::models::SubTask> {
    if payload.title.is_empty() {
        return Err(AppError::BadRequest("Title is required".to_string()));
    }

    let repo = TaskRepository::new(connection::get_database(&*state));
    repo.create_subtask(&task_id, payload.title).await
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
#[tauri::command]
pub async fn update_subtask(
    state: State<'_, DbState>,
    task_id: String,
    subtask_id: String,
    payload: UpdateSubTaskRequest,
) -> Result<crate::db::models::SubTask> {
    let repo = TaskRepository::new(connection::get_database(&*state));
    repo.update_subtask(&task_id, &subtask_id, payload.title, payload.is_completed)
        .await
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
#[tauri::command]
pub async fn delete_subtask(
    state: State<'_, DbState>,
    task_id: String,
    subtask_id: String,
) -> Result<()> {
    let repo = TaskRepository::new(connection::get_database(&*state));
    repo.delete_subtask(&task_id, &subtask_id).await
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
#[tauri::command]
pub async fn reorder_subtasks(
    state: State<'_, DbState>,
    task_id: String,
    payload: ReorderSubTasksRequest,
) -> Result<serde_json::Value> {
    let repo = TaskRepository::new(connection::get_database(&*state));
    repo.reorder_subtasks(&task_id, payload.sub_task_ids).await?;
    Ok(serde_json::json!({"success": true}))
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
#[tauri::command]
pub async fn add_tags_to_task(
    state: State<'_, DbState>,
    id: String,
    tag_ids: Vec<String>,
) -> Result<crate::db::models::Task> {
    let repo = TaskRepository::new(connection::get_database(&*state));
    repo.add_tags(&id, tag_ids).await?;
    
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", id)))
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
#[tauri::command]
pub async fn remove_tags_from_task(
    state: State<'_, DbState>,
    id: String,
    tag_ids: Vec<String>,
) -> Result<crate::db::models::Task> {
    let repo = TaskRepository::new(connection::get_database(&*state));
    repo.remove_tags(&id, tag_ids).await?;
    
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", id)))
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
#[tauri::command]
pub async fn add_goal_to_task(
    state: State<'_, DbState>,
    id: String,
    payload: AddGoalToTaskRequest,
) -> Result<crate::db::models::Task> {
    let repo = TaskRepository::new(connection::get_database(&*state));
    
    let goal_instance_id = repo.add_goal(&id, &payload.goal_id).await?;
    
    repo.update(
        &id,
        None,
        None,
        None,
        None,
        Some(Some(payload.goal_id.clone())),
        Some(goal_instance_id),
        None,
    )
    .await
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
#[tauri::command]
pub async fn remove_goal_from_task(
    state: State<'_, DbState>,
    id: String,
) -> Result<crate::db::models::Task> {
    let repo = TaskRepository::new(connection::get_database(&*state));
    repo.remove_goal(&id).await?;
    
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", id)))
}
