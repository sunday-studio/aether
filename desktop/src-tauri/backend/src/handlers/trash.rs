use crate::db::{connection, DbState};
use crate::error::{AppError, Result};
use crate::utils::log_restore;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};

/// Get all trashed tasks
#[utoipa::path(
    get,
    path = "/v1/trash/tasks",
    tag = "Trash",
    responses(
        (status = 200, description = "List of deleted tasks", body = Vec<crate::db::models::Task>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_trashed_tasks(State(state): State<DbState>) -> Result<impl IntoResponse> {
    let conn = connection::get_database(&state).connect().map_err(|e| AppError::LibSQL(e))?;
    
    let mut rows = conn
        .query(
            "SELECT id, title, description, is_completed, due_date, goal_instance_id, goal_id, created_at, updated_at, deleted_at 
             FROM tasks 
             WHERE deleted_at IS NOT NULL 
             ORDER BY deleted_at DESC",
            libsql::params![],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

    let mut tasks = Vec::new();
    
    while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
        // Use the repository's row_to_task helper if available, or parse manually
        let id: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
        let title: String = row.get(1).map_err(|e| AppError::LibSQL(e))?;
        let description: Option<String> = row.get(2).map_err(|e| AppError::LibSQL(e))?;
        let is_completed: i64 = row.get(3).map_err(|e| AppError::LibSQL(e))?;
        let due_date_str: Option<String> = row.get(4).map_err(|e| AppError::LibSQL(e))?;
        let goal_instance_id: Option<String> = row.get(5).map_err(|e| AppError::LibSQL(e))?;
        let goal_id: Option<String> = row.get(6).map_err(|e| AppError::LibSQL(e))?;
        let created_at_str: String = row.get(7).map_err(|e| AppError::LibSQL(e))?;
        let updated_at_str: String = row.get(8).map_err(|e| AppError::LibSQL(e))?;
        let deleted_at_str: Option<String> = row.get(9).map_err(|e| AppError::LibSQL(e))?;

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&chrono::Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid updated_at: {}", e)))?
            .with_timezone(&chrono::Utc);
        let due_date = due_date_str
            .map(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .flatten()
            .map(|dt| dt.with_timezone(&chrono::Utc));
        let deleted_at = deleted_at_str
            .map(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .flatten()
            .map(|dt| dt.with_timezone(&chrono::Utc));

        tasks.push(crate::db::models::Task {
            id,
            title,
            description,
            is_completed: is_completed != 0,
            due_date,
            goal_instance_id,
            goal_id,
            created_at,
            updated_at,
            deleted_at,
        });
    }

    Ok((StatusCode::OK, Json(tasks)))
}

/// Restore a deleted task
#[utoipa::path(
    post,
    path = "/v1/trash/{id}/restore",
    tag = "Trash",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 204, description = "Task restored"),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn restore_task(
    State(state): State<DbState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let conn = connection::get_database(&state).connect().map_err(|e| AppError::LibSQL(e))?;
    
    // Check if task exists and is deleted
    let mut rows = conn
        .query(
            "SELECT id FROM tasks WHERE id = ?1 AND deleted_at IS NOT NULL",
            libsql::params![id.as_str()],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

    if rows.next().await.map_err(|e| AppError::LibSQL(e))?.is_none() {
        return Err(AppError::NotFound(format!("Deleted task {} not found", id)));
    }

    // Restore task by setting deleted_at to NULL
    let now = chrono::Utc::now();
    let updated_at_str = now.to_rfc3339();

    conn.execute(
        "UPDATE tasks SET deleted_at = NULL, updated_at = ?1 WHERE id = ?2",
        libsql::params![updated_at_str, id.as_str()],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    // Log activity
    let db = connection::get_database(&state);
    let _ = log_restore(db, "task".to_string(), id).await;

    Ok(StatusCode::NO_CONTENT)
}
