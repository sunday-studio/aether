//! Platform boundaries for storage, secure secrets, and sync lifecycle.
//!
//! Desktop currently owns the concrete implementations in this module. Keeping
//! the calls behind these small interfaces gives mobile targets a place to
//! provide their own sandboxed paths and secure-storage adapter without
//! changing repositories or the sync protocol.

use std::path::PathBuf;

use tauri::AppHandle;
use tauri_plugin_keyring::KeyringExt;

use crate::error::{AppError, Result};

const APP_QUALIFIER: &str = "com.cas";
const APP_ORGANIZATION: &str = "aether";
const APP_IDENTIFIER: &str = "com.cas.aether";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoragePaths {
    pub app_data: PathBuf,
    pub media: PathBuf,
    pub cache: PathBuf,
    pub models: PathBuf,
    pub logs: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceIdentityStore {
    /// The stable sync device id lives in the local database metadata.
    DatabaseMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncLifecycle {
    /// Desktop sync runs while the app has an active focused window.
    ForegroundWindow,
}

/// The platform services that the application core is allowed to depend on.
pub trait PlatformCapabilities {
    fn storage_paths(&self) -> Result<StoragePaths>;
    fn device_identity_store(&self) -> DeviceIdentityStore;
    fn sync_lifecycle(&self) -> SyncLifecycle;
}

/// The secure-secret surface used by sync credentials.
///
/// Mobile may replace this Tauri keyring adapter with its platform-native
/// secure storage while preserving the caller contract.
pub trait SecureSecretStore {
    fn set(
        &self,
        app: &AppHandle,
        service: &str,
        account: &str,
        secret: &str,
    ) -> std::result::Result<(), String>;
    fn get(
        &self,
        app: &AppHandle,
        service: &str,
        account: &str,
    ) -> std::result::Result<Option<String>, String>;
    fn delete(
        &self,
        app: &AppHandle,
        service: &str,
        account: &str,
    ) -> std::result::Result<(), String>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DesktopPlatform;

impl DesktopPlatform {
    fn project_dirs(&self) -> Result<directories::ProjectDirs> {
        directories::ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_IDENTIFIER).ok_or_else(
            || {
                AppError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Failed to resolve desktop application directories",
                ))
            },
        )
    }

    fn legacy_media_dir(&self) -> Result<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME")
                .map_err(|_| AppError::Internal("HOME environment variable not set".to_string()))?;
            return Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Aether")
                .join("media"));
        }

        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME")
                .map_err(|_| AppError::Internal("HOME environment variable not set".to_string()))?;
            return Ok(PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("aether")
                .join("media"));
        }

        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA").map_err(|_| {
                AppError::Internal("APPDATA environment variable not set".to_string())
            })?;
            return Ok(PathBuf::from(appdata).join("Aether").join("media"));
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        Ok(PathBuf::from(".").join("media"))
    }
}

impl PlatformCapabilities for DesktopPlatform {
    fn storage_paths(&self) -> Result<StoragePaths> {
        let dirs = self.project_dirs()?;
        let app_data = dirs.data_local_dir().to_path_buf();
        let logs = if cfg!(debug_assertions) {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("diagnostics")
        } else {
            app_data.join("diagnostics")
        };

        Ok(StoragePaths {
            media: self.legacy_media_dir()?,
            models: app_data.join("models"),
            cache: dirs.cache_dir().to_path_buf(),
            app_data,
            logs,
        })
    }

    fn device_identity_store(&self) -> DeviceIdentityStore {
        DeviceIdentityStore::DatabaseMetadata
    }

    fn sync_lifecycle(&self) -> SyncLifecycle {
        SyncLifecycle::ForegroundWindow
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TauriKeyringSecretStore;

impl SecureSecretStore for TauriKeyringSecretStore {
    fn set(
        &self,
        app: &AppHandle,
        service: &str,
        account: &str,
        secret: &str,
    ) -> std::result::Result<(), String> {
        app.keyring()
            .set_password(service, account, secret)
            .map_err(|error| error.to_string())
    }

    fn get(
        &self,
        app: &AppHandle,
        service: &str,
        account: &str,
    ) -> std::result::Result<Option<String>, String> {
        app.keyring()
            .get_password(service, account)
            .map_err(|error| error.to_string())
    }

    fn delete(
        &self,
        app: &AppHandle,
        service: &str,
        account: &str,
    ) -> std::result::Result<(), String> {
        app.keyring()
            .delete_password(service, account)
            .map_err(|error| error.to_string())
    }
}

pub fn desktop() -> DesktopPlatform {
    DesktopPlatform
}

pub fn secure_secrets() -> TauriKeyringSecretStore {
    TauriKeyringSecretStore
}

#[cfg(test)]
mod tests {
    use super::{desktop, DeviceIdentityStore, PlatformCapabilities, SyncLifecycle};

    #[test]
    fn desktop_capabilities_keep_device_identity_and_sync_lifecycle_explicit() {
        let platform = desktop();

        assert_eq!(
            platform.device_identity_store(),
            DeviceIdentityStore::DatabaseMetadata
        );
        assert_eq!(platform.sync_lifecycle(), SyncLifecycle::ForegroundWindow);
    }

    #[test]
    fn desktop_storage_paths_are_scoped_to_separate_capabilities() {
        let paths = desktop().storage_paths().expect("desktop paths resolve");

        assert!(paths.app_data.is_absolute());
        assert!(paths.media.is_absolute());
        assert!(paths.cache.is_absolute());
        assert!(paths.models.starts_with(&paths.app_data));
        assert_ne!(paths.logs, paths.media);
    }

    #[test]
    fn desktop_media_path_preserves_the_existing_storage_location() {
        let media = desktop()
            .storage_paths()
            .expect("desktop paths resolve")
            .media;

        #[cfg(target_os = "macos")]
        assert!(media.ends_with("Library/Application Support/Aether/media"));

        #[cfg(target_os = "linux")]
        assert!(media.ends_with(".local/share/aether/media"));

        #[cfg(target_os = "windows")]
        assert!(media.ends_with("Aether\\media"));
    }
}
