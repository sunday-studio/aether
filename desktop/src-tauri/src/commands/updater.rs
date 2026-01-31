//! Tauri commands for the auto-updater functionality.

use crate::updater::{self, UpdateInfo, UpdateManager, UpdatePreferences};
use tauri::{AppHandle, State};
use tauri_plugin_updater::UpdaterExt;

/// Check for available updates
#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
    manager: State<'_, UpdateManager>,
) -> Result<Option<UpdateInfo>, String> {
    manager.record_check().await;

    let info = updater::check_for_updates(&app).await?;

    // Filter out skipped versions
    if let Some(ref update_info) = info {
        if manager.is_version_skipped(&update_info.latest_version).await {
            return Ok(None);
        }
    }

    Ok(info)
}

/// Download and install the available update
#[tauri::command]
pub async fn download_and_install_update(app: AppHandle) -> Result<(), String> {
    let update = app
        .updater()
        .map_err(|e| format!("Failed to get updater: {}", e))?
        .check()
        .await
        .map_err(|e| format!("Failed to check for updates: {}", e))?;

    let update = update.ok_or_else(|| "No update available".to_string())?;

    // Download the update
    let mut downloaded = 0;
    let bytes = update
        .download(
            |chunk_length, content_length| {
                downloaded += chunk_length;
                if let Some(total) = content_length {
                    tracing::debug!(
                        "[UPDATER] Download progress: {} / {} bytes",
                        downloaded,
                        total
                    );
                }
            },
            || {
                tracing::debug!("[UPDATER] Download chunk received");
            },
        )
        .await
        .map_err(|e| format!("Failed to download update: {}", e))?;

    tracing::info!("[UPDATER] Download complete, installing...");

    // Install the update (this will restart the app)
    update
        .install(bytes)
        .map_err(|e| format!("Failed to install update: {}", e))?;

    // Request app restart
    app.restart();
}

/// Skip a specific version
#[tauri::command]
pub async fn skip_update_version(
    manager: State<'_, UpdateManager>,
    version: String,
) -> Result<(), String> {
    manager.skip_version(version).await;
    Ok(())
}

/// Get update preferences
#[tauri::command]
pub async fn get_update_preferences(
    manager: State<'_, UpdateManager>,
) -> Result<UpdatePreferences, String> {
    Ok(manager.get_preferences().await)
}

/// Set update preferences
#[tauri::command]
pub async fn set_update_preferences(
    manager: State<'_, UpdateManager>,
    preferences: UpdatePreferences,
) -> Result<(), String> {
    manager.set_preferences(preferences).await;
    Ok(())
}

/// Get the current app version
#[tauri::command]
pub async fn get_app_version(app: AppHandle) -> Result<String, String> {
    Ok(app.package_info().version.to_string())
}
