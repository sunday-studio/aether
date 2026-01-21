use crate::db::connection;
use crate::db::DbState;
use crate::error::{AppError, Result};
use crate::settings;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct GetSettingQuery {
    pub key: String,
}

#[derive(Serialize, ToSchema)]
pub struct SettingResponse {
    pub key: String,
    pub value: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct SetSettingRequest {
    pub key: String,
    pub value: String,
}

/// Get a setting value
#[utoipa::path(
    get,
    path = "/v1/settings",
    tag = "Settings",
    params(
        ("key" = String, Query, description = "Setting key")
    ),
    responses(
        (status = 200, description = "Setting value", body = SettingResponse),
        (status = 400, description = "Bad request - key is required"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_setting(
    State(state): State<DbState>,
    Query(params): Query<GetSettingQuery>,
) -> Result<impl IntoResponse> {
    if params.key.is_empty() {
        return Err(AppError::BadRequest("Setting key is required".to_string()));
    }

    let database = connection::get_database(&state);
    let value = settings::get_setting(database, &params.key).await?;

    Ok((
        StatusCode::OK,
        Json(SettingResponse {
            key: params.key,
            value,
        }),
    ))
}

/// Set a setting value
#[utoipa::path(
    post,
    path = "/v1/settings",
    tag = "Settings",
    request_body = SetSettingRequest,
    responses(
        (status = 200, description = "Setting updated successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn set_setting(
    State(state): State<DbState>,
    Json(payload): Json<SetSettingRequest>,
) -> Result<impl IntoResponse> {
    if payload.key.is_empty() {
        return Err(AppError::BadRequest("Setting key is required".to_string()));
    }

    let database = connection::get_database(&state);
    settings::set_setting(database, &payload.key, &payload.value).await?;

    Ok((StatusCode::OK, Json(serde_json::json!({"success": true}))))
}
