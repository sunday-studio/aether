use crate::commands::params::{EmptyPathParams, EmptyQueryParams, EmptyRequest, SettingQueryParams};
use crate::db::connection;
use crate::error::{AppError, Result};
use crate::handlers::settings::{AllSettingsResponse, SettingResponse, SetSettingRequest};
use crate::settings;
use std::collections::HashMap;
use tauri::State;

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
#[tauri::command]
pub async fn get_setting(
    state: State<'_, crate::DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<SettingQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<SettingResponse> {
    let params = query_params.ok_or_else(|| AppError::BadRequest("Query parameters are required".to_string()))?;
    if params.key.is_empty() {
        return Err(AppError::BadRequest("Setting key is required".to_string()));
    }

    let database = connection::get_database(&*state);
    let value = settings::get_setting(database, &params.key).await?;

    Ok(SettingResponse { key: params.key, value })
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
#[tauri::command]
pub async fn get_all_settings(
    state: State<'_, crate::DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<AllSettingsResponse> {
    let database = connection::get_database(&*state);
    let settings = settings::get_all_settings(database).await?;

    let settings_map: HashMap<String, String> = settings.into_iter().collect();
    let settings_response = AllSettingsResponse::from(settings_map);

    Ok(settings_response)
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
#[tauri::command]
pub async fn set_setting(
    state: State<'_, crate::DbState>,
    request_data: Option<SetSettingRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<()> {
    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    if request.key.is_empty() {
        return Err(AppError::BadRequest("Setting key is required".to_string()));
    }

    let database = connection::get_database(&*state);
    settings::set_setting(database, &request.key, &request.value).await?;

    Ok(())
}
