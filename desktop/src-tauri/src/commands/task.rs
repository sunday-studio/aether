use crate::db::{connection, DbState, TaskRepository};
use crate::error::{AppError, Result};
use crate::utils::{
    log_complete, log_create, log_delete, log_goal_operation, log_reorder, log_tag_operation,
    log_update,
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
#[tauri::command(rename_all = "camelCase")]
pub async fn create_task(
    state: State<'_, DbState>,
    title: String,
    description: Option<String>,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    goal_id: Option<String>,
    tag_ids: Option<Vec<String>>,
) -> Result<crate::db::models::Task> {
    if title.is_empty() {
        return Err(AppError::BadRequest("Title is required".to_string()));
    }

    let db = connection::get_database(&*state);
    let repo = TaskRepository::new(db.clone());
    
    let task = repo
        .create(
            title,
            description,
            due_date,
            goal_id.clone(),
            None, // goal_instance_id - will be set in Milestone 5
        )
        .await?;

    if let Some(tag_ids) = tag_ids {
        if !tag_ids.is_empty() {
            repo.add_tags(&task.id, tag_ids).await?;
        }
    }

    // Log activity
    if let Err(e) = log_create(db, "task".to_string(), task.id.clone()).await {
        tracing::warn!("Failed to log task creation activity: {}", e);
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
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }
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
#[tauri::command(rename_all = "camelCase")]
pub async fn update_task(
    state: State<'_, DbState>,
    id: String,
    title: Option<String>,
    description: Option<Option<String>>,
    due_date: Option<Option<chrono::DateTime<chrono::Utc>>>,
    is_completed: Option<bool>,
    goal_id: Option<Option<String>>,
    tag_ids: Option<Vec<String>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<crate::db::models::Task> {
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }

    let db = connection::get_database(&*state);
    let repo = TaskRepository::new(db.clone());
    
    // Get current task to check completion status
    let old_task = repo.find_by_id(&id).await?;
    let was_completed = old_task.as_ref().map(|t| t.is_completed).unwrap_or(false);
    
    let task = repo
        .update(
            &id,
            title,
            description,
            due_date,
            is_completed,
            goal_id,
            None, // goal_instance_id - will be set in Milestone 5
            updated_at,
        )
        .await?;

    if let Some(tag_ids) = tag_ids {
        let current_task = repo.find_by_id(&id).await?;
        if current_task.is_some() {
            repo.add_tags(&id, tag_ids).await?;
        }
    }

    // Log activity - check if completion changed
    if let Some(new_completed) = is_completed {
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
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }
    let db = connection::get_database(&*state);
    let repo = TaskRepository::new(db.clone());
    repo.delete(&id).await?;
    
    // Log activity
    if let Err(e) = log_delete(db, "task".to_string(), id.clone()).await {
        tracing::warn!("Failed to log task deletion activity for task {}: {}", id, e);
    }
    
    Ok(())
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
    if task_id.is_empty() {
        return Err(AppError::BadRequest("Task ID is required".to_string()));
    }
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
#[tauri::command(rename_all = "camelCase")]
pub async fn create_subtask(
    state: State<'_, DbState>,
    task_id: String,
    title: String,
) -> Result<crate::db::models::SubTask> {
    if task_id.is_empty() {
        return Err(AppError::BadRequest("Task ID is required".to_string()));
    }
    if title.is_empty() {
        return Err(AppError::BadRequest("Title is required".to_string()));
    }

    let db = connection::get_database(&*state);
    let repo = TaskRepository::new(db.clone());
    let subtask = repo.create_subtask(&task_id, title).await?;
    
    // Log activity
    if let Err(e) = log_create(db, "subtask".to_string(), subtask.id.clone()).await {
        tracing::warn!("Failed to log subtask creation activity: {}", e);
    }
    
    Ok(subtask)
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
#[tauri::command(rename_all = "camelCase")]
pub async fn update_subtask(
    state: State<'_, DbState>,
    task_id: String,
    subtask_id: String,
    title: Option<String>,
    is_completed: Option<bool>,
) -> Result<crate::db::models::SubTask> {
    if task_id.is_empty() {
        return Err(AppError::BadRequest("Task ID is required".to_string()));
    }
    if subtask_id.is_empty() {
        return Err(AppError::BadRequest("Subtask ID is required".to_string()));
    }

    let db = connection::get_database(&*state);
    let repo = TaskRepository::new(db.clone());
    
    // Get current subtask to check completion status
    let subtasks = repo.find_subtasks(&task_id).await?;
    let old_subtask = subtasks.iter().find(|s| s.id == subtask_id);
    let was_completed = old_subtask.map(|s| s.is_completed).unwrap_or(false);
    
    let subtask = repo.update_subtask(&task_id, &subtask_id, title, is_completed)
        .await?;
    
    // Log activity - check if completion changed
    if let Some(new_completed) = is_completed {
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
    
    Ok(subtask)
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
    if task_id.is_empty() {
        return Err(AppError::BadRequest("Task ID is required".to_string()));
    }
    if subtask_id.is_empty() {
        return Err(AppError::BadRequest("Subtask ID is required".to_string()));
    }
    let db = connection::get_database(&*state);
    let repo = TaskRepository::new(db.clone());
    repo.delete_subtask(&task_id, &subtask_id).await?;
    
    // Log activity
    if let Err(e) = log_delete(db, "subtask".to_string(), subtask_id.clone()).await {
        tracing::warn!("Failed to log subtask deletion activity for subtask {}: {}", subtask_id, e);
    }
    
    Ok(())
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
#[tauri::command(rename_all = "camelCase")]
pub async fn reorder_subtasks(
    state: State<'_, DbState>,
    task_id: String,
    sub_task_ids: Vec<String>,
) -> Result<serde_json::Value> {
    let db = connection::get_database(&*state);
    let repo = TaskRepository::new(db.clone());
    repo.reorder_subtasks(&task_id, sub_task_ids).await?;
    
    // Log activity - reorder is logged for the task
    if let Err(e) = log_reorder(db, "task".to_string(), task_id.clone()).await {
        tracing::warn!("Failed to log subtask reorder activity for task {}: {}", task_id, e);
    }
    
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
    let db = connection::get_database(&*state);
    let repo = TaskRepository::new(db.clone());
    repo.add_tags(&id, tag_ids).await?;
    
    // Log activity
    if let Err(e) = log_tag_operation(db.clone(), "add_tags", "task".to_string(), id.clone()).await {
        tracing::warn!("Failed to log add_tags activity for task {}: {}", id, e);
    }
    
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
    let db = connection::get_database(&*state);
    let repo = TaskRepository::new(db.clone());
    repo.remove_tags(&id, tag_ids).await?;
    
    // Log activity
    if let Err(e) = log_tag_operation(db.clone(), "remove_tags", "task".to_string(), id.clone()).await {
        tracing::warn!("Failed to log remove_tags activity for task {}: {}", id, e);
    }
    
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
#[tauri::command(rename_all = "camelCase")]
pub async fn add_goal_to_task(
    state: State<'_, DbState>,
    id: String,
    goal_id: String,
) -> Result<crate::db::models::Task> {
    let db = connection::get_database(&*state);
    let repo = TaskRepository::new(db.clone());
    
    let goal_instance_id = repo.add_goal(&id, &goal_id).await?;
    
    let task = repo.update(
        &id,
        None,
        None,
        None,
        None,
        Some(Some(goal_id.clone())),
        Some(goal_instance_id),
        None,
    )
    .await?;
    
    // Log activity
    if let Err(e) = log_goal_operation(db, "add_goal", "task".to_string(), task.id.clone()).await {
        tracing::warn!("Failed to log add_goal activity for task {}: {}", task.id, e);
    }
    
    Ok(task)
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
    let db = connection::get_database(&*state);
    let repo = TaskRepository::new(db.clone());
    repo.remove_goal(&id).await?;
    
    // Log activity
    if let Err(e) = log_goal_operation(db.clone(), "remove_goal", "task".to_string(), id.clone()).await {
        tracing::warn!("Failed to log remove_goal activity for task {}: {}", id, e);
    }
    
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", id)))
}
