//! Sync engine: configure, sync (pull+apply+push), status, disconnect.

use crate::db::connection::get_database;
use crate::db::DbState;
use crate::error::{AppError, Result};
use crate::settings;
use crate::sync::apply;
use crate::sync::metadata;
use crate::sync::pull;
use crate::sync::push;
use crate::sync::register;
use serde::Serialize;
use std::sync::Mutex;
use tauri::AppHandle;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SyncStatus {
    pub connected: bool,
    pub pending_changes: u32,
    pub last_sync: Option<i64>,
    /// True when server_url is in memory but passphrase is not (e.g. after app restart).
    pub needs_passphrase: bool,
}

pub struct SyncEngine {
    db: DbState,
    server_url: Mutex<Option<String>>,
    passphrase: Mutex<Option<String>>,
    is_syncing: Mutex<bool>,
}

const SERVICE_NAME: &str = "com.aether.sync";
const PASSPHRASE_KEY: &str = "encryption_passphrase";

impl SyncEngine {
    pub fn new(db: DbState) -> Self {
        Self {
            db,
            server_url: Mutex::new(None),
            passphrase: Mutex::new(None),
            is_syncing: Mutex::new(false),
        }
    }

    /// Store passphrase in OS keychain
    fn store_passphrase(&self, app: &AppHandle, passphrase: &str) -> Result<()> {
        use tauri_plugin_keyring::KeyringExt;
        app.keyring()
            .set_password(SERVICE_NAME, PASSPHRASE_KEY, passphrase)
            .map_err(|e| AppError::Sync(format!("failed to store passphrase in keychain: {}", e)))?;
        tracing::info!("[SYNC] Passphrase stored in keychain");
        Ok(())
    }

    /// Retrieve passphrase from OS keychain
    fn get_passphrase(&self, app: &AppHandle) -> Result<Option<String>> {
        use tauri_plugin_keyring::KeyringExt;
        match app.keyring().get_password(SERVICE_NAME, PASSPHRASE_KEY) {
            Ok(Some(pass)) => {
                tracing::info!("[SYNC] Passphrase retrieved from keychain");
                Ok(Some(pass))
            }
            Ok(None) => {
                tracing::debug!("[SYNC] No passphrase found in keychain");
                Ok(None)
            }
            Err(e) => {
                tracing::warn!("[SYNC] Failed to retrieve passphrase from keychain: {}", e);
                Ok(None)
            }
        }
    }

    /// Clear passphrase from OS keychain
    fn clear_passphrase(&self, app: &AppHandle) -> Result<()> {
        use tauri_plugin_keyring::KeyringExt;
        match app.keyring().delete_password(SERVICE_NAME, PASSPHRASE_KEY) {
            Ok(()) => {
                tracing::info!("[SYNC] Passphrase cleared from keychain");
                Ok(())
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("No matching entry") || err_str.contains("not found") {
                    tracing::debug!("[SYNC] No passphrase to clear from keychain");
                    Ok(())
                } else {
                    tracing::warn!("[SYNC] Failed to clear passphrase from keychain: {}", e);
                    Ok(()) // Don't fail on clear errors
                }
            }
        }
    }

    /// Load server_url from persisted metadata into memory and attempt to load passphrase from keychain.
    /// Call after app start so status().connected is true when sync was previously configured.
    pub async fn hydrate(&self, app: &AppHandle) -> Result<()> {
        tracing::info!("[SYNC] Hydrating sync configuration from metadata");
        let db = get_database(&self.db);
        if let Some(url) = metadata::get_server_url(db.as_ref()).await? {
            *self.server_url.lock().unwrap() = Some(url.clone());
            tracing::info!("[SYNC] Loaded server URL from metadata: {}", url);

            // Attempt to load passphrase from keychain
            if let Some(passphrase) = self.get_passphrase(app)? {
                // Verify passphrase against stored key_check
                match metadata::verify_key(db.as_ref(), &passphrase).await {
                    Ok(_) => {
                        *self.passphrase.lock().unwrap() = Some(passphrase);
                        tracing::info!("[SYNC] Passphrase loaded from keychain and verified");
                    }
                    Err(e) => {
                        tracing::warn!("[SYNC] Passphrase from keychain failed verification: {}", e);
                        // Clear invalid passphrase from keychain
                        let _ = self.clear_passphrase(app);
                    }
                }
            } else {
                tracing::info!("[SYNC] No passphrase found in keychain");
            }
        } else {
            tracing::info!("[SYNC] No server URL found in metadata");
        }
        Ok(())
    }

    /// Configure sync: server URL and passphrase. On first run, fetches salt from server.
    pub async fn configure(&self, app: &AppHandle, server_url: String, passphrase: String) -> Result<()> {
        tracing::info!("[SYNC] Configuring sync with server URL: {}", server_url);
        let db = get_database(&self.db);

        // Check if we have a cached salt locally
        if metadata::get_key_salt(&db).await?.is_none() {
            // No local salt - fetch from server (first connect or after disconnect)
            tracing::info!("[SYNC] No local salt found, fetching from server");
            let salt = metadata::fetch_server_salt(&server_url).await?;
            tracing::info!("[SYNC] Salt fetched from server, configuring key");
            metadata::configure_key(&db, &passphrase, &salt).await?;
        } else {
            // Local salt exists - verify passphrase against it
            tracing::info!("[SYNC] Verifying passphrase against cached salt");
            metadata::verify_key(&db, &passphrase).await?;
        }

        metadata::set_server_url(&db, &server_url).await?;
        *self.server_url.lock().unwrap() = Some(server_url.clone());
        *self.passphrase.lock().unwrap() = Some(passphrase.clone());
        // Store passphrase in keychain
        self.store_passphrase(app, &passphrase)?;
        tracing::info!("[SYNC] Configuration complete");
        Ok(())
    }

    /// Run pull, apply, then push.
    pub async fn sync(&self) -> Result<SyncStatus> {
        tracing::info!("[SYNC] Starting sync operation");
        let (url, pass) = {
            let u = self.server_url.lock().unwrap().clone();
            let p = self.passphrase.lock().unwrap().clone();
            (u, p)
        };
        let url = url.ok_or_else(|| {
            tracing::error!("[SYNC] Sync not configured: server URL missing");
            AppError::Sync("sync not configured".into())
        })?;
        let pass = pass.ok_or_else(|| {
            tracing::error!("[SYNC] Sync not configured: passphrase missing");
            AppError::Sync("sync not configured".into())
        })?;

        tracing::info!("[SYNC] Server URL: {}", url);

        let already = {
            let mut g = self.is_syncing.lock().unwrap();
            let v = *g;
            if !v {
                *g = true;
            }
            v
        };
        if already {
            tracing::warn!("[SYNC] Sync already in progress, returning current status");
            return self.status().await;
        }

        let db = get_database(&self.db);
        let key = match metadata::verify_key(&db, &pass).await {
            Ok(k) => {
                tracing::info!("[SYNC] Passphrase verified successfully");
                k
            }
            Err(e) => {
                tracing::error!("[SYNC] Passphrase verification failed: {}", e);
                *self.is_syncing.lock().unwrap() = false;
                return Err(e);
            }
        };

        // Check outbox before sync
        let pending_before: i64 = {
            let conn = db.connect().map_err(AppError::LibSQL)?;
            let mut rows = conn
                .query("SELECT COUNT(*) FROM _sync_outbox", libsql::params![])
                .await
                .map_err(AppError::LibSQL)?;
            let result = if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
                row.get(0).unwrap_or(0)
            } else {
                0
            };
            // Connection is dropped here when it goes out of scope
            result
        };
        tracing::info!("[SYNC] Pending changes in outbox before sync: {}", pending_before);

        let device_id = metadata::get_device_id(&db).await?;
        let device_hostname = metadata::get_device_hostname(&db).await.unwrap_or_else(|_| "unknown".to_string());

        if let Err(e) = register::register_with_server(&url, &device_id, &device_hostname, &pass).await {
            if e.to_string().contains("401") || e.to_string().contains("wrong passphrase") {
                *self.is_syncing.lock().unwrap() = false;
                return Err(e);
            }
            tracing::warn!("[SYNC] Register failed (server may not require it): {}", e);
        }

        // Pull
        tracing::info!("[SYNC-PULL] Starting pull from server");
        let (envelopes, ts) = match pull::pull(&db, &key, &url, &device_id).await {
            Ok((e, t)) => {
                tracing::info!("[SYNC-PULL] Pull successful: received {} changes, timestamp: {}", e.len(), t);
                (e, t)
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("401") || err_str.contains("not registered") {
                    tracing::info!("[SYNC] Pull 401, re-registering and retrying");
                    let _ = register::register_with_server(&url, &device_id, &device_hostname, &pass).await;
                    match pull::pull(&db, &key, &url, &device_id).await {
                        Ok((e, t)) => (e, t),
                        Err(e2) => {
                            tracing::error!("[SYNC] Pull failed after retry: {}", e2);
                            *self.is_syncing.lock().unwrap() = false;
                            return Err(e2);
                        }
                    }
                } else {
                    tracing::error!("[SYNC] Pull failed: {}", e);
                    *self.is_syncing.lock().unwrap() = false;
                    return Err(e);
                }
            }
        };

        let media_sync_policy = settings::get_setting(db.clone(), "sync.media_sync_policy")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "on_demand".to_string());
        tracing::info!("[SYNC-PULL] Media sync policy: {}", media_sync_policy);

        let ctx = apply::ApplyCtx {
            base_url: &url,
            key: &key,
            media_sync_policy: &media_sync_policy,
        };

        // Apply with triggers suppressed
        tracing::info!("[SYNC-PULL] Applying {} remote changes", envelopes.len());
        let apply_res = apply::with_suppress_triggers(&db, async {
            let mut applied = 0;
            let mut skipped = 0;
            for e in &envelopes {
                match apply::apply_change(&*db, e, Some(&ctx)).await {
                    Ok(()) => {
                        applied += 1;
                        tracing::debug!("[SYNC] Applied change: {} {} ({:?})", e.entity, e.id, e.op);
                    }
                    Err(err) => {
                        tracing::warn!("[SYNC] Failed to apply change {} {}: {}", e.entity, e.id, err);
                        skipped += 1;
                    }
                }
            }
            tracing::info!("[SYNC] Applied {} changes, skipped {} changes", applied, skipped);
            Ok(())
        })
        .await;

        if let Err(e) = apply_res {
            tracing::error!("[SYNC] Apply phase failed: {}", e);
            *self.is_syncing.lock().unwrap() = false;
            return Err(e);
        }

        metadata::set_last_sync(&db, ts).await?;
        tracing::info!("[SYNC] Last sync timestamp updated to: {}", ts);

        // Push
        tracing::info!("[SYNC] Starting push to server");
        let push_res = push::push(db.clone(), &key, &url, &media_sync_policy).await;
        match push_res {
            Ok(count) => {
                tracing::info!("[SYNC] Push successful: {} changes pushed", count);
            }
            Err(e) => {
                let err_str = e.to_string();
                if (err_str.contains("401") || err_str.contains("not registered")) && !device_id.is_empty() {
                    let _ = register::register_with_server(&url, &device_id, &device_hostname, &pass).await;
                    if let Ok(count) = push::push(db.clone(), &key, &url, &media_sync_policy).await {
                        tracing::info!("[SYNC] Push successful after re-register: {} changes pushed", count);
                    } else {
                        tracing::error!("[SYNC] Push failed after retry: {}", e);
                    }
                } else {
                    tracing::error!("[SYNC] Push failed: {}", e);
                }
            }
        }

        // Check outbox after sync
        let pending_after: i64 = {
            let conn = db.connect().map_err(AppError::LibSQL)?;
            let mut rows = conn
                .query("SELECT COUNT(*) FROM _sync_outbox", libsql::params![])
                .await
                .map_err(AppError::LibSQL)?;
            let result = if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
                row.get(0).unwrap_or(0)
            } else {
                0
            };
            // Connection is dropped here when it goes out of scope
            result
        };
        tracing::info!("[SYNC] Pending changes in outbox after sync: {}", pending_after);

        *self.is_syncing.lock().unwrap() = false;
        tracing::info!("[SYNC] Sync operation completed");
        self.status().await
    }

    /// Push only (no pull). Use on window blur to flush pending changes.
    pub async fn push_pending(&self) -> Result<()> {
        tracing::info!("[SYNC] Pushing pending changes");
        let (url, pass) = {
            let u = self.server_url.lock().unwrap().clone();
            let p = self.passphrase.lock().unwrap().clone();
            (u, p)
        };
        let Some(url) = url else {
            tracing::debug!("[SYNC] No server URL configured, skipping push");
            return Ok(());
        };
        let Some(pass) = pass else {
            tracing::debug!("[SYNC] No passphrase in memory, skipping push");
            return Ok(());
        };
        let db = get_database(&self.db);
        let key = metadata::verify_key(&db, &pass).await?;
        let media_sync_policy = settings::get_setting(db.clone(), "sync.media_sync_policy")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "on_demand".to_string());
        match push::push(db, &key, &url, &media_sync_policy).await {
            Ok(count) => {
                tracing::info!("[SYNC] Pushed {} pending changes", count);
            }
            Err(e) => {
                tracing::warn!("[SYNC] Failed to push pending changes: {}", e);
            }
        }
        Ok(())
    }

    pub async fn status(&self) -> Result<SyncStatus> {
        let db = get_database(&self.db);
        let conn = db.connect().map_err(AppError::LibSQL)?;

        let connected = self.server_url.lock().unwrap().is_some();
        let needs_passphrase = self.server_url.lock().unwrap().is_some()
            && self.passphrase.lock().unwrap().is_none();

        let mut rows = conn
            .query("SELECT COUNT(*) FROM _sync_outbox", libsql::params![])
            .await
            .map_err(AppError::LibSQL)?;
        let pending: i64 = if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            row.get(0).unwrap_or(0)
        } else {
            0
        };

        let last_sync = metadata::get_last_sync(&db).await.ok().flatten();

        let status = SyncStatus {
            connected,
            pending_changes: pending as u32,
            last_sync,
            needs_passphrase,
        };

        tracing::debug!(
            "[SYNC] Status: connected={}, pending={}, last_sync={:?}, needs_passphrase={}",
            status.connected,
            status.pending_changes,
            status.last_sync,
            status.needs_passphrase
        );

        Ok(status)
    }

    /// Returns true if a sync is currently in progress.
    pub fn is_syncing(&self) -> bool {
        *self.is_syncing.lock().unwrap()
    }

    /// Re-enter passphrase when needs_passphrase is true (e.g. after app restart).
    pub async fn reconnect(&self, passphrase: String) -> Result<()> {
        let url = self.server_url.lock().unwrap().clone();
        let Some(_) = url else {
            return Err(AppError::Sync("sync not configured".into()));
        };
        let db = get_database(&self.db);
        metadata::verify_key(&db, &passphrase).await?;
        *self.passphrase.lock().unwrap() = Some(passphrase);
        Ok(())
    }

    pub async fn disconnect(&self, app: &AppHandle) -> Result<()> {
        // Clear passphrase from keychain
        let _ = self.clear_passphrase(app);
        *self.server_url.lock().unwrap() = None;
        *self.passphrase.lock().unwrap() = None;

        // Clear salt and key_check so next connect fetches fresh from server
        let db = get_database(&self.db);
        metadata::clear_key_material(&db).await?;

        tracing::info!("[SYNC] Disconnected: cleared keychain, salt, and in-memory state");
        Ok(())
    }

    /// If sync is configured, returns the server URL. Used for on-demand media fetch.
    pub fn try_get_url(&self) -> Option<String> {
        self.server_url.lock().unwrap().clone()
    }

    /// If passphrase is in memory, verifies and returns the key. Used for on-demand media decrypt.
    pub async fn try_get_key(&self) -> Option<[u8; 32]> {
        let pass = self.passphrase.lock().unwrap().clone()?;
        let db = get_database(&self.db);
        metadata::verify_key(db.as_ref(), &pass).await.ok()
    }

    /// Get the device ID for this device. Used for WebSocket registration.
    pub async fn get_device_id(&self) -> Result<String> {
        let db = get_database(&self.db);
        metadata::get_device_id(&db).await
    }

    /// Get the device hostname for this device. Used for WebSocket registration.
    pub async fn get_device_hostname(&self) -> Result<String> {
        let db = get_database(&self.db);
        metadata::get_device_hostname(&db).await
    }
}
