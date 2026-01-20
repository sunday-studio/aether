use crate::db::{connection, DbState, GoalRepository};
use crate::error::{AppError, Result};
use crate::handlers::goal::{CreateGoalRequest, UpdateGoalRequest};
use tauri::State;

/// Get all goals
#[utoipa::path(
    get,
    path = "/v1/goals",
    tag = "Goals",
    responses(
        (status = 200, description = "List of all goals", body = Vec<crate::db::models::Goal>),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_goals(state: State<'_, DbState>) -> Result<Vec<crate::db::models::Goal>> {
    let repo = GoalRepository::new(connection::get_database(&*state));
    repo.find_all().await
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
        (status = 200, description = "Goal found", body = crate::db::models::Goal),
        (status = 404, description = "Goal not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_goal_by_id(
    state: State<'_, DbState>,
    id: String,
) -> Result<crate::db::models::Goal> {
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
        (status = 200, description = "Created goal", body = crate::db::models::Goal),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn create_goal(
    state: State<'_, DbState>,
    payload: CreateGoalRequest,
) -> Result<crate::db::models::Goal> {
    if payload.name.is_empty() {
        return Err(AppError::BadRequest("Name is required".to_string()));
    }

    let is_non_recurring = payload.is_non_recurring.unwrap_or(false);

    if !is_non_recurring {
        if payload.recurrence_type.is_none() || payload.recurrence_type.as_ref().unwrap().is_empty() {
            return Err(AppError::BadRequest(
                "recurrenceType is required for recurring goals".to_string(),
            ));
        }
        if payload.recurrence_interval.is_none() {
            return Err(AppError::BadRequest(
                "recurrenceInterval is required for recurring goals".to_string(),
            ));
        }
        if payload.recurrence_anchor.is_none() {
            return Err(AppError::BadRequest(
                "recurrenceAnchor is required for recurring goals".to_string(),
            ));
        }
    }

    if is_non_recurring {
        if payload.recurrence_type.is_some()
            || payload.recurrence_interval.is_some()
            || payload.recurrence_anchor.is_some()
        {
            return Err(AppError::BadRequest(
                "recurrence fields must be null for non-recurring goals".to_string(),
            ));
        }
    }

    let timezone = "UTC".to_string();

    let repo = GoalRepository::new(connection::get_database(&*state));
    let goal = repo
        .create(
            payload.name,
            payload.description,
            is_non_recurring,
            payload.recurrence_type,
            payload.recurrence_interval,
            payload.recurrence_anchor,
            payload.recurrence_meta,
            timezone,
        )
        .await?;

    if !payload.tag_ids.is_empty() {
        repo.add_tags(&goal.id, payload.tag_ids).await?;
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
        (status = 200, description = "Updated goal", body = crate::db::models::Goal),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Goal not found"),
        (status = 409, description = "Conflict: Goal was modified by another device"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn update_goal(
    state: State<'_, DbState>,
    id: String,
    payload: UpdateGoalRequest,
) -> Result<crate::db::models::Goal> {
    let repo = GoalRepository::new(connection::get_database(&*state));
    let goal = repo
        .update(
            &id,
            payload.name,
            payload.description,
            payload.is_non_recurring,
            payload.recurrence_type,
            payload.recurrence_interval,
            payload.recurrence_anchor,
            payload.recurrence_meta,
            payload.updated_at,
        )
        .await?;

    if let Some(tag_ids) = payload.tag_ids {
        repo.remove_tags(&id, vec![]).await?;
        if !tag_ids.is_empty() {
            repo.add_tags(&id, tag_ids).await?;
        }
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
pub async fn delete_goal(state: State<'_, DbState>, id: String) -> Result<()> {
    let repo = GoalRepository::new(connection::get_database(&*state));
    repo.delete(&id).await
}

/// Get goal instances for a goal
#[utoipa::path(
    get,
    path = "/v1/goals/{goalId}/instances",
    tag = "GoalInstances",
    params(
        ("goalId" = String, Path, description = "Goal ID")
    ),
    responses(
        (status = 200, description = "List of goal instances", body = Vec<crate::db::models::GoalInstance>),
        (status = 404, description = "Goal not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_goal_instances(
    state: State<'_, DbState>,
    goal_id: String,
) -> Result<Vec<crate::db::models::GoalInstance>> {
    let repo = GoalRepository::new(connection::get_database(&*state));
    repo.find_instances(&goal_id).await
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
        (status = 200, description = "Current goal instance", body = crate::db::models::GoalInstance),
        (status = 404, description = "Goal not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_current_goal_instance(
    state: State<'_, DbState>,
    goal_id: String,
) -> Result<crate::db::models::GoalInstance> {
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
        (status = 200, description = "Tags added to goal", body = crate::db::models::Goal),
        (status = 404, description = "Goal or tag not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn add_tags_to_goal(
    state: State<'_, DbState>,
    id: String,
    tag_ids: Vec<String>,
) -> Result<crate::db::models::Goal> {
    let repo = GoalRepository::new(connection::get_database(&*state));
    repo.add_tags(&id, tag_ids).await?;
    
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
        (status = 200, description = "Tags removed from goal", body = crate::db::models::Goal),
        (status = 404, description = "Goal not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn remove_tags_from_goal(
    state: State<'_, DbState>,
    id: String,
    tag_ids: Vec<String>,
) -> Result<crate::db::models::Goal> {
    let repo = GoalRepository::new(connection::get_database(&*state));
    repo.remove_tags(&id, tag_ids).await?;
    
    repo.find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Goal {} not found", id)))
}
