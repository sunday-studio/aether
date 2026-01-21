use crate::db::{connection, ActivityRepository, DbState};
use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tauri::State;

/// Get activities grouped by date with detailed breakdowns
#[utoipa::path(
    get,
    path = "/v1/activities",
    tag = "Activities",
    params(
        ("start_date" = Option<String>, Query, description = "Start date (ISO 8601 format, defaults to 1 year ago)"),
        ("end_date" = Option<String>, Query, description = "End date (ISO 8601 format, defaults to now)")
    ),
    responses(
        (status = 200, description = "Activities grouped by date"),
        (status = 400, description = "Bad request - invalid date format"),
        (status = 500, description = "Internal server error")
    )
)]
#[tauri::command]
pub async fn get_activities(
    state: State<'_, DbState>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<HashMap<String, HashMap<String, HashMap<String, i64>>>> {
    let db = connection::get_database(&*state);
    let repo = ActivityRepository::new(db);

    // Parse dates if provided, otherwise use defaults
    let start_date_parsed = if let Some(start_str) = start_date {
        Some(
            DateTime::parse_from_rfc3339(&start_str)
                .map_err(|e| AppError::BadRequest(format!(
                    "Invalid start_date format: {}. Expected ISO 8601 format (e.g., 2024-01-15T00:00:00Z)",
                    e
                )))?
                .with_timezone(&Utc),
        )
    } else {
        None
    };

    let end_date_parsed = if let Some(end_str) = end_date {
        Some(
            DateTime::parse_from_rfc3339(&end_str)
                .map_err(|e| AppError::BadRequest(format!(
                    "Invalid end_date format: {}. Expected ISO 8601 format (e.g., 2024-01-15T00:00:00Z)",
                    e
                )))?
                .with_timezone(&Utc),
        )
    } else {
        None
    };

    repo.get_by_date_range(start_date_parsed, end_date_parsed).await
}
