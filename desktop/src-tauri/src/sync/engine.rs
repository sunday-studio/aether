//! Sync engine: configure, sync (pull+apply+push), status, disconnect.

use crate::db::connection::get_database;
use crate::db::DbState;
use crate::error::{AppError, Result};
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
}

pub struct SyncEngine {
    db: DbState,
    server_url: Mutex<Option<String>>,
    passphrase: Mutex<Option<String>>,
}

impl SyncEngine {
    pub fn new(db: DbState) -> Self {
        Self {
            db,
            server_url: Mutex::new(None),
            passphrase: Mutex::new(None),
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

        let db = get_database(&self.db);
        let key = metadata::verify_key(&db, &pass).await?;

        // Pull
        let (envelopes, ts) = pull::pull(&db, &key, &url).await?;

        // Apply with triggers suppressed
        apply::with_suppress_triggers(&db, async {
            for e in &envelopes {
                apply::apply_change(&*db, e).await?;
            }
            Ok(())
        })
        .await?;

        metadata::set_last_sync(&db, ts).await?;

        // Push
        let _ = push::push(db, &key, &url).await;

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
        let _ = push::push(db, &key, &url).await;
        Ok(())
    }

    pub async fn status(&self) -> Result<SyncStatus> {
        let db = get_database(&self.db);
        let conn = db.connect().map_err(AppError::LibSQL)?;

        let connected = self.server_url.lock().unwrap().is_some();

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
        })
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
