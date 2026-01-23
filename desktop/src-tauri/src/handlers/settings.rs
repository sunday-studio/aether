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
use std::collections::HashMap;
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

#[derive(Serialize)]
#[serde(transparent)]
pub struct AllSettingsResponse(HashMap<String, String>);

impl ToSchema<'_> for AllSettingsResponse {
    fn schema() -> (&'static str, utoipa::openapi::RefOr<utoipa::openapi::Schema>) {
        (
            "AllSettingsResponse",
            utoipa::openapi::ObjectBuilder::new()
                .additional_properties(
                    utoipa::openapi::ObjectBuilder::new()
                        .schema_type(utoipa::openapi::SchemaType::String)
                        .into(),
                )
                .into(),
        )
    }
}

impl From<HashMap<String, String>> for AllSettingsResponse {
    fn from(map: HashMap<String, String>) -> Self {
        AllSettingsResponse(map)
    }
}

impl From<AllSettingsResponse> for HashMap<String, String> {
    fn from(response: AllSettingsResponse) -> Self {
        response.0
    }
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

/// Get all settings
#[utoipa::path(
    get,
    path = "/v1/settings/all",
    tag = "Settings",
    responses(
        (status = 200, description = "All settings", body = AllSettingsResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_all_settings(
    State(state): State<DbState>,
) -> Result<impl IntoResponse> {
    let database = connection::get_database(&state);
    let settings = settings::get_all_settings(database).await?;

    let settings_map: HashMap<String, String> = settings.into_iter().collect();
    let settings_response = AllSettingsResponse(settings_map);

    Ok((
        StatusCode::OK,
        Json(settings_response),
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
