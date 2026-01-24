//! Sync engine: configure, sync (pull+apply+push), status, disconnect.

use crate::db::connection::get_database;
use crate::db::DbState;
use crate::error::{AppError, Result};
use crate::settings;
use crate::sync::apply;
use crate::sync::metadata;
use crate::sync::pull;
use crate::sync::push;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
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

impl SyncEngine {
    pub fn new(db: DbState) -> Self {
        Self {
            db,
            server_url: Mutex::new(None),
            passphrase: Mutex::new(None),
            is_syncing: Mutex::new(false),
        }
    }

    /// Load server_url from persisted metadata into memory. Call after app start so
    /// status().connected is true when sync was previously configured. Passphrase
    /// is not stored; user must re-enter to run sync.
    pub async fn hydrate_from_metadata(&self) -> Result<()> {
        let db = get_database(&self.db);
        if let Some(url) = metadata::get_server_url(db.as_ref()).await? {
            *self.server_url.lock().unwrap() = Some(url);
        }
        Ok(())
    }

    /// Configure sync: server URL and passphrase. On first run, derives and stores key_salt/key_check.
    pub async fn configure(&self, server_url: String, passphrase: String) -> Result<()> {
        let db = get_database(&self.db);
        if metadata::get_key_salt(&db).await?.is_none() {
            metadata::configure_key(&db, &passphrase).await?;
        } else {
            metadata::verify_key(&db, &passphrase).await?;
        }
        metadata::set_server_url(&db, &server_url).await?;
        *self.server_url.lock().unwrap() = Some(server_url);
        *self.passphrase.lock().unwrap() = Some(passphrase);
        Ok(())
    }

    /// Run pull, apply, then push.
    pub async fn sync(&self) -> Result<SyncStatus> {
        let (url, pass) = {
            let u = self.server_url.lock().unwrap().clone();
            let p = self.passphrase.lock().unwrap().clone();
            (u, p)
        };
        let url = url.ok_or_else(|| AppError::Sync("sync not configured".into()))?;
        let pass = pass.ok_or_else(|| AppError::Sync("sync not configured".into()))?;

        let already = {
            let mut g = self.is_syncing.lock().unwrap();
            let v = *g;
            if !v {
                *g = true;
            }
            v
        };
        if already {
            return self.status().await;
        }

        let db = get_database(&self.db);
        let key = metadata::verify_key(&db, &pass).await?;

        // Pull
        let (envelopes, ts) = pull::pull(&db, &key, &url).await?;

        let media_sync_policy = settings::get_setting(db.clone(), "sync.media_sync_policy")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "on_demand".to_string());

        let ctx = apply::ApplyCtx {
            base_url: &url,
            key: &key,
            media_sync_policy: &media_sync_policy,
        };

        // Apply with triggers suppressed
        let apply_res = apply::with_suppress_triggers(&db, async {
            for e in &envelopes {
                apply::apply_change(&*db, e, Some(&ctx)).await?;
            }
            Ok(())
        })
        .await;

        if let Err(e) = apply_res {
            *self.is_syncing.lock().unwrap() = false;
            return Err(e);
        }

        metadata::set_last_sync(&db, ts).await?;

        // Push
        let _ = push::push(db, &key, &url, &media_sync_policy).await;

        *self.is_syncing.lock().unwrap() = false;
        self.status().await
    }

    /// Push only (no pull). Use on window blur to flush pending changes.
    pub async fn push_pending(&self) -> Result<()> {
        let (url, pass) = {
            let u = self.server_url.lock().unwrap().clone();
            let p = self.passphrase.lock().unwrap().clone();
            (u, p)
        };
        let Some(url) = url else { return Ok(()); };
        let Some(pass) = pass else { return Ok(()); };
        let db = get_database(&self.db);
        let key = metadata::verify_key(&db, &pass).await?;
        let media_sync_policy = settings::get_setting(db.clone(), "sync.media_sync_policy")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "on_demand".to_string());
        let _ = push::push(db, &key, &url, &media_sync_policy).await;
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

        Ok(SyncStatus {
            connected,
            pending_changes: pending as u32,
            last_sync,
            needs_passphrase,
        })
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

    pub async fn disconnect(&self) -> Result<()> {
        *self.server_url.lock().unwrap() = None;
        *self.passphrase.lock().unwrap() = None;
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
}
