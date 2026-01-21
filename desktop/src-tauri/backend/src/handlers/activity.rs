use crate::db::{connection, ActivityRepository, DbState};
use crate::error::Result;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct ActivityQueryParams {
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
}

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
        (status = 200, description = "Activities grouped by date", body = HashMap<String, HashMap<String, HashMap<String, i64>>>),
        (status = 400, description = "Bad request - invalid date format"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_activities(
    State(state): State<DbState>,
    Query(params): Query<ActivityQueryParams>,
) -> Result<impl IntoResponse> {
    let db = connection::get_database(&state);
    let repo = ActivityRepository::new(db);

    // Parse dates if provided, otherwise use defaults
    let start_date = if let Some(start_str) = params.start_date {
        Some(
            DateTime::parse_from_rfc3339(&start_str)
                .map_err(|e| crate::error::AppError::BadRequest(format!(
                    "Invalid start_date format: {}. Expected ISO 8601 format (e.g., 2024-01-15T00:00:00Z)",
                    e
                )))?
                .with_timezone(&Utc),
        )
    } else {
        None
    };

    let end_date = if let Some(end_str) = params.end_date {
        Some(
            DateTime::parse_from_rfc3339(&end_str)
                .map_err(|e| crate::error::AppError::BadRequest(format!(
                    "Invalid end_date format: {}. Expected ISO 8601 format (e.g., 2024-01-15T00:00:00Z)",
                    e
                )))?
                .with_timezone(&Utc),
        )
    } else {
        None
    };

    let activities = repo.get_by_date_range(start_date, end_date).await?;

    Ok((StatusCode::OK, Json(activities)))
}
