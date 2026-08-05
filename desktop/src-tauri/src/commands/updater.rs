//! Tauri commands for the auto-updater functionality.

use crate::updater::{self, UpdateCheckStatus, UpdateInfo, UpdateManager, UpdatePreferences};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_updater::UpdaterExt;

const GITHUB_RELEASES_URL: &str =
    "https://api.github.com/repos/sunday-studio/aether/releases?per_page=20";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateProgress {
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    percentage: f64,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    published_at: Option<String>,
    prerelease: bool,
    draft: bool,
    html_url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseHistoryItem {
    tag_name: String,
    name: String,
    notes: String,
    published_at: Option<String>,
    url: String,
}

/// List published GitHub releases for the What's New screen.
///
/// The stable updater feed remains the sole source for downloadable updates.
#[tauri::command]
pub async fn get_release_history() -> Result<Vec<ReleaseHistoryItem>, String> {
    let response = reqwest::Client::new()
        .get(GITHUB_RELEASES_URL)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .header(reqwest::header::USER_AGENT, "Aether Desktop")
        .send()
        .await
        .map_err(|error| format!("Failed to load release history: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Failed to load release history: {error}"))?;

    let releases = response
        .json::<Vec<GitHubRelease>>()
        .await
        .map_err(|error| format!("Failed to read release history: {error}"))?;

    Ok(published_releases(releases))
}

fn published_releases(releases: Vec<GitHubRelease>) -> Vec<ReleaseHistoryItem> {
    releases
        .into_iter()
        .filter(|release| !release.prerelease && !release.draft)
        .map(|release| ReleaseHistoryItem {
            name: release.name.unwrap_or_else(|| release.tag_name.clone()),
            tag_name: release.tag_name,
            notes: release.body.unwrap_or_default(),
            published_at: release.published_at,
            url: release.html_url,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{published_releases, GitHubRelease};

    fn release(tag_name: &str, prerelease: bool, draft: bool) -> GitHubRelease {
        GitHubRelease {
            tag_name: tag_name.to_string(),
            name: None,
            body: Some("Preview notes".to_string()),
            published_at: Some("2026-08-05T00:00:00Z".to_string()),
            prerelease,
            draft,
            html_url: format!("https://github.com/sunday-studio/aether/releases/tag/{tag_name}"),
        }
    }

    #[test]
    fn only_returns_published_stable_releases() {
        let releases = published_releases(vec![
            release("v0.1.6-alpha.1", true, false),
            release("v0.1.5", false, false),
            release("v0.1.6-beta.1", true, true),
        ]);

        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].tag_name, "v0.1.5");
    }
}

/// Check for available updates
#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
    manager: State<'_, UpdateManager>,
) -> Result<Option<UpdateInfo>, String> {
    let result = updater::check_for_updates(&app).await;
    manager.record_check(result.is_err()).await;

    let info = result?;
    let _ = app.emit("update-check-succeeded", manager.get_check_status().await);

    // Filter out skipped versions
    if let Some(ref update_info) = info {
        if manager
            .is_version_skipped(&update_info.latest_version)
            .await
        {
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
    let progress_app = app.clone();
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
                let percentage = content_length
                    .map(|total| {
                        if total == 0 {
                            0.0
                        } else {
                            ((downloaded as f64 / total as f64) * 100.0).min(100.0)
                        }
                    })
                    .unwrap_or(0.0);
                let _ = progress_app.emit(
                    "update-download-progress",
                    UpdateProgress {
                        downloaded_bytes: downloaded as u64,
                        total_bytes: content_length,
                        percentage,
                    },
                );
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

/// Get the last successful update-feed check for the settings UI.
#[tauri::command]
pub async fn get_update_check_status(
    manager: State<'_, UpdateManager>,
) -> Result<UpdateCheckStatus, String> {
    Ok(manager.get_check_status().await)
}

/// Get the current app version
#[tauri::command]
pub async fn get_app_version(app: AppHandle) -> Result<String, String> {
    Ok(app.package_info().version.to_string())
}
