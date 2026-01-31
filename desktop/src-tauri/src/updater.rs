//! Auto-updater module for managing application updates.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tokio::sync::RwLock;

/// Minimum time between update checks (30 minutes)
const CHECK_COOLDOWN_SECS: u64 = 30 * 60;

/// Information about an available update
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub changelog: String,
    pub published_at: Option<String>,
}

/// User preferences for update behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePreferences {
    pub auto_check: bool,
    pub auto_download: bool,
    pub skipped_versions: Vec<String>,
}

impl Default for UpdatePreferences {
    fn default() -> Self {
        Self {
            auto_check: true,
            auto_download: false,
            skipped_versions: Vec::new(),
        }
    }
}

/// Manages update checking with cooldown and preferences
pub struct UpdateManager {
    last_check: Arc<RwLock<Option<Instant>>>,
    preferences: Arc<RwLock<UpdatePreferences>>,
}

impl UpdateManager {
    pub fn new() -> Self {
        Self {
            last_check: Arc::new(RwLock::new(None)),
            preferences: Arc::new(RwLock::new(UpdatePreferences::default())),
        }
    }

    /// Check if enough time has passed since last update check
    pub async fn should_check(&self) -> bool {
        let prefs = self.preferences.read().await;
        if !prefs.auto_check {
            return false;
        }
        drop(prefs);

        let last = self.last_check.read().await;
        match *last {
            Some(instant) => instant.elapsed() >= Duration::from_secs(CHECK_COOLDOWN_SECS),
            None => true,
        }
    }

    /// Record that an update check was performed
    pub async fn record_check(&self) {
        let mut last = self.last_check.write().await;
        *last = Some(Instant::now());
    }

    /// Check if a version should be skipped
    pub async fn is_version_skipped(&self, version: &str) -> bool {
        let prefs = self.preferences.read().await;
        prefs.skipped_versions.contains(&version.to_string())
    }

    /// Add a version to the skip list
    pub async fn skip_version(&self, version: String) {
        let mut prefs = self.preferences.write().await;
        if !prefs.skipped_versions.contains(&version) {
            prefs.skipped_versions.push(version);
        }
    }

    /// Get current preferences
    pub async fn get_preferences(&self) -> UpdatePreferences {
        self.preferences.read().await.clone()
    }

    /// Update preferences
    pub async fn set_preferences(&self, new_prefs: UpdatePreferences) {
        let mut prefs = self.preferences.write().await;
        *prefs = new_prefs;
    }
}

impl Default for UpdateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Check for updates using the Tauri updater plugin
pub async fn check_for_updates(app: &AppHandle) -> Result<Option<UpdateInfo>, String> {
    use tauri_plugin_updater::UpdaterExt;

    let current_version = app.package_info().version.to_string();

    let update = app
        .updater()
        .map_err(|e| format!("Failed to get updater: {}", e))?
        .check()
        .await
        .map_err(|e| format!("Failed to check for updates: {}", e))?;

    match update {
        Some(update) => {
            let info = UpdateInfo {
                current_version,
                latest_version: update.version.clone(),
                changelog: update.body.clone().unwrap_or_default(),
                published_at: update.date.as_ref().map(|d| d.to_string()),
            };
            Ok(Some(info))
        }
        None => Ok(None),
    }
}
