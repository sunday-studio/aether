use crate::db::connection;
use crate::error::{AppError, Result};
use crate::handlers::settings::{AllSettingsResponse, SetSettingRequest, SettingResponse};
use crate::settings;
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
    key: String,
) -> Result<SettingResponse> {
    if key.is_empty() {
        return Err(AppError::BadRequest("Setting key is required".to_string()));
    }

    let database = connection::get_database(&*state);
    let value = settings::get_setting(database, &key).await?;

    Ok(SettingResponse { key, value })
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
) -> Result<AllSettingsResponse> {
    let database = connection::get_database(&*state);
    let settings = settings::get_all_settings(database).await?;

    let settings_response: Vec<SettingResponse> = settings
        .into_iter()
        .map(|(key, value)| SettingResponse {
            key,
            value: Some(value),
        })
        .collect();

    Ok(AllSettingsResponse {
        settings: settings_response,
    })
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
    key: String,
    value: String,
) -> Result<()> {
    if key.is_empty() {
        return Err(AppError::BadRequest("Setting key is required".to_string()));
    }

    let database = connection::get_database(&*state);
    settings::set_setting(database, &key, &value).await?;

    Ok(())
}
