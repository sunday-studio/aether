use crate::db::{connection, DbState, GoalRepository};
use crate::error::{AppError, Result};
use crate::utils::{log_create, log_delete, log_tag_operation, log_update};
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
pub struct CreateGoalRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_non_recurring: Option<bool>,
    #[serde(default)]
    pub recurrence_type: Option<String>,
    #[serde(default)]
    pub recurrence_interval: Option<i32>,
    #[serde(default)]
    pub recurrence_anchor: Option<chrono::DateTime<Utc>>,
    #[serde(default)]
    pub recurrence_meta: Option<serde_json::Value>,
    #[serde(default)]
    pub tag_ids: Vec<String>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGoalRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<Option<String>>,
    #[serde(default)]
    pub is_non_recurring: Option<bool>,
    #[serde(default)]
    pub recurrence_type: Option<Option<String>>,
    #[serde(default)]
    pub recurrence_interval: Option<Option<i32>>,
    #[serde(default)]
    pub recurrence_anchor: Option<Option<chrono::DateTime<Utc>>>,
    #[serde(default)]
    pub recurrence_meta: Option<Option<serde_json::Value>>,
    #[serde(default)]
    pub tag_ids: Option<Vec<String>>,
    #[serde(default)]
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

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
pub async fn get_goals(State(state): State<DbState>) -> Result<impl IntoResponse> {
    let repo = GoalRepository::new(connection::get_database(&state));
    let goals = repo.find_all().await?;
    Ok((StatusCode::OK, Json(goals)))
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
pub async fn get_goal_by_id(
    State(state): State<DbState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let repo = GoalRepository::new(connection::get_database(&state));
    match repo.find_by_id(&id).await? {
        Some(goal) => Ok((StatusCode::OK, Json(goal))),
        None => Err(AppError::NotFound(format!("Goal {} not found", id))),
    }
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
pub async fn create_goal(
    State(state): State<DbState>,
    Json(payload): Json<CreateGoalRequest>,
) -> Result<impl IntoResponse> {
    if payload.name.is_empty() {
        return Err(AppError::BadRequest("Name is required".to_string()));
    }

    // Determine if goal is non-recurring (default to false if not provided)
    let is_non_recurring = payload.is_non_recurring.unwrap_or(false);

    // Validation: for recurring goals, require recurrence fields
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

    // Validation: for non-recurring goals, recurrence fields should be nil
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

    // Get user's current timezone from Settings (default to UTC if not found)
    // For now, we'll default to UTC - can be enhanced later to read from settings
    let timezone = "UTC".to_string();

    let db = connection::get_database(&state);
    let repo = GoalRepository::new(db.clone());
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

    // Add tags if provided
    if !payload.tag_ids.is_empty() {
        repo.add_tags(&goal.id, payload.tag_ids).await?;
    }

    // Log activity
    if let Err(e) = log_create(db, "goal".to_string(), goal.id.clone()).await {
        tracing::warn!("Failed to log goal creation activity: {}", e);
    }

    Ok((StatusCode::OK, Json(goal)))
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
pub async fn update_goal(
    State(state): State<DbState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateGoalRequest>,
) -> Result<impl IntoResponse> {
    let db = connection::get_database(&state);
    let repo = GoalRepository::new(db.clone());
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

    // Update tags if provided
    if let Some(tag_ids) = payload.tag_ids {
        // For simplicity, we'll replace all tags
        // First remove all existing tags, then add new ones
        // This could be optimized in the future
        repo.remove_tags(&id, vec![]).await?; // Remove all
        if !tag_ids.is_empty() {
            repo.add_tags(&id, tag_ids).await?;
        }
    }

    // Log activity
    if let Err(e) = log_update(db, "goal".to_string(), goal.id.clone()).await {
        tracing::warn!("Failed to log goal update activity: {}", e);
    }

    Ok((StatusCode::OK, Json(goal)))
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
pub async fn delete_goal(
    State(state): State<DbState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let db = connection::get_database(&state);
    let repo = GoalRepository::new(db.clone());
    repo.delete(&id).await?;
    
    // Log activity
    if let Err(e) = log_delete(db, "goal".to_string(), id.clone()).await {
        tracing::warn!("Failed to log goal deletion activity for goal {}: {}", id, e);
    }
    
    Ok(StatusCode::NO_CONTENT)
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
pub async fn get_goal_instances(
    State(state): State<DbState>,
    Path(goal_id): Path<String>,
) -> Result<impl IntoResponse> {
    let repo = GoalRepository::new(connection::get_database(&state));
    let instances = repo.find_instances(&goal_id).await?;
    Ok((StatusCode::OK, Json(instances)))
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
pub async fn get_current_goal_instance(
    State(state): State<DbState>,
    Path(goal_id): Path<String>,
) -> Result<impl IntoResponse> {
    let repo = GoalRepository::new(connection::get_database(&state));
    let instance = repo.get_or_create_current_instance(&goal_id).await?;
    Ok((StatusCode::OK, Json(instance)))
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
pub async fn add_tags_to_goal(
    State(state): State<DbState>,
    Path(id): Path<String>,
    Json(tag_ids): Json<Vec<String>>,
) -> Result<impl IntoResponse> {
    let db = connection::get_database(&state);
    let repo = GoalRepository::new(db.clone());
    repo.add_tags(&id, tag_ids).await?;
    
    // Log activity
    if let Err(e) = log_tag_operation(db.clone(), "add_tags", "goal".to_string(), id.clone()).await {
        tracing::warn!("Failed to log add_tags activity for goal {}: {}", id, e);
    }
    
    // Return updated goal
    let goal = repo.find_by_id(&id).await?
        .ok_or_else(|| AppError::NotFound(format!("Goal {} not found", id)))?;
    Ok((StatusCode::OK, Json(goal)))
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
pub async fn remove_tags_from_goal(
    State(state): State<DbState>,
    Path(id): Path<String>,
    Json(tag_ids): Json<Vec<String>>,
) -> Result<impl IntoResponse> {
    let db = connection::get_database(&state);
    let repo = GoalRepository::new(db.clone());
    repo.remove_tags(&id, tag_ids).await?;
    
    // Log activity
    if let Err(e) = log_tag_operation(db.clone(), "remove_tags", "goal".to_string(), id.clone()).await {
        tracing::warn!("Failed to log remove_tags activity for goal {}: {}", id, e);
    }
    
    // Return updated goal
    let goal = repo.find_by_id(&id).await?
        .ok_or_else(|| AppError::NotFound(format!("Goal {} not found", id)))?;
    Ok((StatusCode::OK, Json(goal)))
}
