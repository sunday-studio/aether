use crate::commands::params::{
    EmptyPathParams, EmptyQueryParams, EmptyRequest, GoalIdPathParams, IdPathParams,
    PaginationQueryParams,
};
use crate::db::models::{Goal, GoalInstance, GoalInstanceWithTasks};
use crate::db::{connection, DbState, GoalRepository, TaskRepository};
use crate::error::{AppError, Result};
use crate::handlers::common::PaginationResponse;
use crate::handlers::goal::{CreateGoalRequest, UpdateGoalRequest};
use crate::utils::{log_create, log_delete, log_tag_operation, log_update};
use serde::Deserialize;
use tauri::State;
use utoipa::ToSchema;

/// Request to add tags to a goal
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct AddTagsToGoalRequest {
    pub tag_ids: Vec<String>,
}

/// Request to remove tags from a goal
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RemoveTagsFromGoalRequest {
    pub tag_ids: Vec<String>,
}

/// Get all goals
#[utoipa::path(
    get,
    path = "/v1/goals",
    tag = "Goals",
    params(
        ("limit" = Option<u32>, Query, description = "Number of goals per page (max 1000)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination")
    ),
    responses(
        (status = 200, description = "Paginated list of goals", body = PaginatedGoals),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_goals(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<PaginationQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<PaginationResponse<Goal>> {
    let _guard = connection::with_db_access(&*state).await;
    let params = query_params.unwrap_or_default();
    let repo = GoalRepository::new(connection::get_database(&*state));
    let (goals, next_cursor, has_more) = repo
        .find_all(params.normalize_limit(), params.cursor)
        .await?;
    Ok(PaginationResponse::new(goals, next_cursor, has_more))
}

/// Get goal by ID
#[utoipa::path(
    get,
    path = "/v1/goals/{id}",
    tag = "Goals",
    params(
        ("id" = String, Path, description = "Goal ID")
    ),
    responses(
        (status = 200, description = "Goal found", body = Goal),
        (status = 404, description = "Goal not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_goal_by_id(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<Goal> {
    let _guard = connection::with_db_access(&*state).await;
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    let repo = GoalRepository::new(connection::get_database(&*state));
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Goal {} not found", id)))
}

/// Create a new goal
#[utoipa::path(
    post,
    path = "/v1/goals",
    tag = "Goals",
    request_body = CreateGoalRequest,
    responses(
        (status = 200, description = "Created goal", body = Goal),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn create_goal(
    state: State<'_, DbState>,
    request_data: Option<CreateGoalRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<Goal> {
    let _guard = connection::with_db_access(&*state).await;
    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    if request.name.is_empty() {
        return Err(AppError::BadRequest("Name is required".to_string()));
    }

    let is_non_recurring = request.is_non_recurring.unwrap_or(false);

    if !is_non_recurring {
        if request.recurrence_type.is_none() || request.recurrence_type.as_ref().unwrap().is_empty() {
            return Err(AppError::BadRequest(
                "recurrenceType is required for recurring goals".to_string(),
            ));
        }
        if request.recurrence_interval.is_none() {
            return Err(AppError::BadRequest(
                "recurrenceInterval is required for recurring goals".to_string(),
            ));
        }
        if request.recurrence_anchor.is_none() {
            return Err(AppError::BadRequest(
                "recurrenceAnchor is required for recurring goals".to_string(),
            ));
        }
    }

    if is_non_recurring {
        if request.recurrence_type.is_some()
            || request.recurrence_interval.is_some()
            || request.recurrence_anchor.is_some()
        {
            return Err(AppError::BadRequest(
                "recurrence fields must be null for non-recurring goals".to_string(),
            ));
        }
    }

    let timezone = "UTC".to_string();

    let db = connection::get_database(&*state);
    let repo = GoalRepository::new(db.clone());
    let goal = repo
        .create(
            request.name,
            request.description,
            is_non_recurring,
            request.recurrence_type,
            request.recurrence_interval,
            request.recurrence_anchor,
            request.recurrence_meta,
            timezone,
        )
        .await?;

    if !request.tag_ids.is_empty() {
        repo.add_tags(&goal.id, request.tag_ids).await?;
    }

    // Log activity
    if let Err(e) = log_create(db, "goal".to_string(), goal.id.clone()).await {
        tracing::warn!("Failed to log goal creation activity: {}", e);
    }

    Ok(goal)
}

/// Update a goal
#[utoipa::path(
    put,
    path = "/v1/goals/{id}",
    tag = "Goals",
    params(
        ("id" = String, Path, description = "Goal ID")
    ),
    request_body = UpdateGoalRequest,
    responses(
        (status = 200, description = "Updated goal", body = Goal),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Goal not found"),
        (status = 409, description = "Conflict: Goal was modified by another device"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn update_goal(
    state: State<'_, DbState>,
    request_data: Option<UpdateGoalRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<Goal> {
    let _guard = connection::with_db_access(&*state).await;
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }

    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;

    let db = connection::get_database(&*state);
    let repo = GoalRepository::new(db.clone());
    let goal = repo
        .update(
            &id,
            request.name,
            request.description,
            request.is_non_recurring,
            request.recurrence_type,
            request.recurrence_interval,
            request.recurrence_anchor,
            request.recurrence_meta,
            request.updated_at,
        )
        .await?;

    if let Some(tag_ids) = request.tag_ids {
        repo.remove_tags(&id, vec![]).await?;
        if !tag_ids.is_empty() {
            repo.add_tags(&id, tag_ids).await?;
        }
    }

    // Log activity
    if let Err(e) = log_update(db, "goal".to_string(), goal.id.clone()).await {
        tracing::warn!("Failed to log goal update activity: {}", e);
    }

    Ok(goal)
}

/// Delete a goal
#[utoipa::path(
    delete,
    path = "/v1/goals/{id}",
    tag = "Goals",
    params(
        ("id" = String, Path, description = "Goal ID")
    ),
    responses(
        (status = 204, description = "Goal deleted"),
        (status = 404, description = "Goal not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn delete_goal(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<()> {
    let _guard = connection::with_db_access(&*state).await;
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    if id.is_empty() {
        return Err(AppError::BadRequest("ID is required".to_string()));
    }
    let db = connection::get_database(&*state);
    let repo = GoalRepository::new(db.clone());
    repo.delete(&id).await?;
    
    // Log activity
    if let Err(e) = log_delete(db, "goal".to_string(), id.clone()).await {
        tracing::warn!("Failed to log goal deletion activity for goal {}: {}", id, e);
    }
    
    Ok(())
}

/// Get goal instances for a goal (with tasks per instance for goal view)
#[utoipa::path(
    get,
    path = "/v1/goals/{goalId}/instances",
    tag = "GoalInstances",
    params(
        ("goalId" = String, Path, description = "Goal ID"),
        ("limit" = Option<u32>, Query, description = "Number of instances per page (max 1000)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination")
    ),
    responses(
        (status = 200, description = "Paginated list of goal instances with tasks", body = PaginatedGoalInstancesWithTasks),
        (status = 404, description = "Goal not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_goal_instances(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<PaginationQueryParams>,
    path_params: Option<GoalIdPathParams>,
) -> Result<PaginationResponse<GoalInstanceWithTasks>> {
    let _guard = connection::with_db_access(&*state).await;
    let goal_id = path_params
        .and_then(|p| Some(p.goal_id))
        .ok_or_else(|| AppError::BadRequest("Goal ID is required".to_string()))?;
    let params = query_params.unwrap_or_default();
    let db = connection::get_database(&*state);
    let goal_repo = GoalRepository::new(db.clone());
    let task_repo = TaskRepository::new(db);
    let (instances, next_cursor, has_more) = goal_repo
        .find_instances(&goal_id, params.normalize_limit(), params.cursor)
        .await?;
    let mut instances_with_tasks = Vec::with_capacity(instances.len());
    for instance in instances {
        let tasks = task_repo.find_by_goal_instance_id(&instance.id).await?;
        let tasks_with_subtasks = task_repo.with_subtasks(tasks).await?;
        instances_with_tasks.push(GoalInstanceWithTasks {
            id: instance.id,
            goal_id: instance.goal_id,
            period_start: instance.period_start,
            period_end: instance.period_end,
            status: instance.status,
            created_at: instance.created_at,
            tasks: tasks_with_subtasks,
        });
    }
    Ok(PaginationResponse::new(
        instances_with_tasks,
        next_cursor,
        has_more,
    ))
}

/// Get or create current goal instance
#[utoipa::path(
    get,
    path = "/v1/goals/{goalId}/instances/current",
    tag = "GoalInstances",
    params(
        ("goalId" = String, Path, description = "Goal ID")
    ),
    responses(
        (status = 200, description = "Current goal instance", body = GoalInstance),
        (status = 404, description = "Goal not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_current_goal_instance(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<GoalIdPathParams>,
) -> Result<GoalInstance> {
    let _guard = connection::with_db_access(&*state).await;
    let goal_id = path_params
        .and_then(|p| Some(p.goal_id))
        .ok_or_else(|| AppError::BadRequest("Goal ID is required".to_string()))?;
    let repo = GoalRepository::new(connection::get_database(&*state));
    repo.get_or_create_current_instance(&goal_id).await
}

/// Add tags to a goal
#[utoipa::path(
    post,
    path = "/v1/goals/{id}/tags",
    tag = "Goals",
    params(
        ("id" = String, Path, description = "Goal ID")
    ),
    request_body = Vec<String>,
    responses(
        (status = 200, description = "Tags added to goal", body = Goal),
        (status = 404, description = "Goal or tag not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn add_tags_to_goal(
    state: State<'_, DbState>,
    request_data: Option<AddTagsToGoalRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<Goal> {
    let _guard = connection::with_db_access(&*state).await;
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    let db = connection::get_database(&*state);
    let repo = GoalRepository::new(db.clone());
    repo.add_tags(&id, request.tag_ids).await?;
    
    // Log activity
    if let Err(e) = log_tag_operation(db.clone(), "add_tags", "goal".to_string(), id.clone()).await {
        tracing::warn!("Failed to log add_tags activity for goal {}: {}", id, e);
    }
    
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Goal {} not found", id)))
}

/// Remove tags from a goal
#[utoipa::path(
    delete,
    path = "/v1/goals/{id}/tags",
    tag = "Goals",
    params(
        ("id" = String, Path, description = "Goal ID")
    ),
    request_body = Vec<String>,
    responses(
        (status = 200, description = "Tags removed from goal", body = Goal),
        (status = 404, description = "Goal not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn remove_tags_from_goal(
    state: State<'_, DbState>,
    request_data: Option<RemoveTagsFromGoalRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<Goal> {
    let _guard = connection::with_db_access(&*state).await;
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    let db = connection::get_database(&*state);
    let repo = GoalRepository::new(db.clone());
    repo.remove_tags(&id, request.tag_ids).await?;
    
    // Log activity
    if let Err(e) = log_tag_operation(db.clone(), "remove_tags", "goal".to_string(), id.clone()).await {
        tracing::warn!("Failed to log remove_tags activity for goal {}: {}", id, e);
    }
    
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Goal {} not found", id)))
}
