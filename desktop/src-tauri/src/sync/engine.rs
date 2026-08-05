//! Sync engine: configure, sync (pull+apply+push), status, disconnect.

use crate::db::connection::{get_database, with_db_access};
use crate::db::DbState;
use crate::error::{AppError, Result};
use crate::platform::{secure_secrets, SecureSecretStore};
use crate::settings;
use crate::sync::{apply, metadata, ordering, pull, push, register};
use crate::utils::performance_ledger::record_error;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;
use tauri::AppHandle;
use tokio::sync::Notify;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SyncStatus {
    pub connected: bool,
    pub pending_changes: u32,
    pub last_sync: Option<i64>,
    pub needs_passphrase: bool,
    pub server_url: Option<String>,
}

pub struct SyncEngine {
    db: DbState,
    server_url: Mutex<Option<String>>,
    passphrase: Mutex<Option<String>>,
    is_syncing: Mutex<bool>,
    ws_url_configured: Arc<Notify>,
}

const SERVICE_NAME: &str = "com.aether.sync";
const PASSPHRASE_KEY: &str = "encryption_passphrase";

async fn apply_pulled_changes(
    db: &libsql::Database,
    envelopes: &[crate::sync::types::ChangeEnvelope],
    next_cursor: Option<&crate::sync::types::PullCursor>,
    ctx: &apply::ApplyCtx<'_>,
) -> Result<()> {
    let apply_started = Instant::now();
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut ordered_envelopes = envelopes.to_vec();
    ordering::sort_for_dependencies(&mut ordered_envelopes);
    apply::with_suppress_triggers(db, async {
        let mut deferred = 0;
        for envelope in &ordered_envelopes {
            if envelope.op == crate::sync::types::ChangeOp::Delete {
                apply::discard_deferred_change(
                    &conn,
                    &envelope.entity,
                    &envelope.id,
                    envelope.updated_at,
                )
                .await?;
            }
            match apply::apply_change_with_conn(&conn, envelope, Some(ctx)).await {
                Ok(()) => {}
                Err(error) if apply::is_foreign_key_failure(&error) => {
                    apply::defer_foreign_key_change(&conn, envelope, &error).await?;
                    deferred += 1;
                    tracing::warn!(
                        "[SYNC-APPLY] Deferring {} {} until its foreign-key parent arrives: {}",
                        envelope.entity,
                        envelope.id,
                        error
                    );
                }
                Err(error) => {
                    return Err(AppError::Sync(format!(
                        "failed to apply change {} {}: {}",
                        envelope.entity, envelope.id, error
                    )));
                }
            }
        }
        let recovered = apply::retry_deferred_foreign_key_changes(&conn, Some(ctx)).await?;
        tracing::debug!(
            "[SYNC-APPLY] deferred_foreign_key_changes={} recovered_deferred_changes={}",
            deferred,
            recovered
        );
        Ok(())
    })
    .await?;

    if let Some(cursor) = next_cursor {
        metadata::set_pull_cursor(db, cursor).await?;
    }

    tracing::info!(
        "[SYNC-TIMING] apply_changes={}ms changes={}",
        apply_started.elapsed().as_millis(),
        envelopes.len()
    );
    Ok(())
}

impl SyncEngine {
    pub fn new(db: DbState) -> Self {
        Self {
            db,
            server_url: Mutex::new(None),
            passphrase: Mutex::new(None),
            is_syncing: Mutex::new(false),
            ws_url_configured: Arc::new(Notify::new()),
        }
    }

    pub async fn wait_for_url_configured(&self) {
        self.ws_url_configured.notified().await;
    }

    fn store_passphrase(&self, app: &AppHandle, passphrase: &str) -> Result<()> {
        secure_secrets()
            .set(app, SERVICE_NAME, PASSPHRASE_KEY, passphrase)
            .map_err(|e| {
                AppError::Sync(format!("failed to store passphrase in keychain: {}", e))
            })?;
        Ok(())
    }

    fn get_passphrase(&self, app: &AppHandle) -> Result<Option<String>> {
        match secure_secrets().get(app, SERVICE_NAME, PASSPHRASE_KEY) {
            Ok(Some(pass)) => Ok(Some(pass)),
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!("[SYNC] Failed to retrieve passphrase from keychain: {}", e);
                Ok(None)
            }
        }
    }

    fn clear_passphrase(&self, app: &AppHandle) -> Result<()> {
        match secure_secrets().delete(app, SERVICE_NAME, PASSPHRASE_KEY) {
            Ok(()) => Ok(()),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("No matching entry") || err_str.contains("not found") {
                    Ok(())
                } else {
                    tracing::warn!("[SYNC] Failed to clear passphrase from keychain: {}", e);
                    Ok(())
                }
            }
        }
    }

    pub async fn hydrate(&self, app: &AppHandle) -> Result<()> {
        let _guard = with_db_access(&self.db).await;
        let db = get_database(&self.db);
        if let Some(url) = metadata::get_server_url(db.as_ref()).await? {
            *self.server_url.lock().unwrap() = Some(url);
            self.ws_url_configured.notify_one();
            if let Some(passphrase) = self.get_passphrase(app)? {
                match metadata::verify_key(db.as_ref(), &passphrase).await {
                    Ok(_) => *self.passphrase.lock().unwrap() = Some(passphrase),
                    Err(e) => {
                        tracing::warn!(
                            "[SYNC] Passphrase from keychain failed verification: {}",
                            e
                        );
                        let _ = self.clear_passphrase(app);
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn configure(
        &self,
        app: &AppHandle,
        server_url: String,
        server_seed_phrase: String,
        sync_passphrase: String,
    ) -> Result<()> {
        tracing::info!("[SYNC] Configuring sync with server URL: {}", server_url);
        let db = get_database(&self.db);

        let (device_id, device_hostname) = {
            let _guard = with_db_access(&self.db).await;
            let device_id = metadata::get_device_id(&db).await?;
            let device_hostname = metadata::get_device_hostname(&db).await?;
            (device_id, device_hostname)
        };
        let enrollment = register::register_with_server(
            &server_url,
            &device_id,
            &device_hostname,
            &server_seed_phrase,
        )
        .await?;

        {
            let _guard = with_db_access(&self.db).await;
            let salt = metadata::decode_server_salt(&enrollment.salt)?;
            metadata::configure_key(&db, &sync_passphrase, &salt).await?;
            metadata::set_server_url(&db, &server_url).await?;
            metadata::set_device_token(&db, &enrollment.device_token).await?;
            metadata::clear_pull_cursor(&db).await?;
        }

        *self.server_url.lock().unwrap() = Some(server_url);
        *self.passphrase.lock().unwrap() = Some(sync_passphrase.clone());
        self.ws_url_configured.notify_one();
        self.store_passphrase(app, &sync_passphrase)?;
        Ok(())
    }

    pub async fn sync(&self) -> Result<SyncStatus> {
        let (url, pass) = {
            let u = self.server_url.lock().unwrap().clone();
            let p = self.passphrase.lock().unwrap().clone();
            (u, p)
        };
        let Some(url) = url else {
            tracing::debug!("[SYNC] Sync skipped because no server URL is configured");
            return self.status().await;
        };
        let Some(pass) = pass else {
            tracing::debug!("[SYNC] Sync skipped because no passphrase is loaded");
            return self.status().await;
        };

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

        let sync_started = Instant::now();
        let result = self.sync_inner(&url, &pass).await;
        let sync_elapsed_ms = sync_started.elapsed().as_millis();
        if let Err(error) = &result {
            record_error(
                "sync.run",
                "Sync operation failed",
                json!({
                    "error": error.to_string(),
                    "elapsed_ms": sync_elapsed_ms,
                }),
            );
        }
        tracing::info!(
            "[SYNC-TIMING] sync_total={}ms success={}",
            sync_elapsed_ms,
            result.is_ok()
        );
        *self.is_syncing.lock().unwrap() = false;
        result?;
        self.status().await
    }

    async fn sync_inner(&self, url: &str, passphrase: &str) -> Result<()> {
        let db = get_database(&self.db);
        let (key, device_id, device_token, media_sync_policy) = {
            let _guard = with_db_access(&self.db).await;
            let key = metadata::verify_key(&db, passphrase).await?;
            let device_id = metadata::get_device_id(&db).await?;
            let device_token = metadata::get_device_token(&db)
                .await?
                .ok_or_else(|| AppError::Sync("device token missing".into()))?;
            let media_sync_policy = settings::get_setting(db.clone(), "sync.media_sync_policy")
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| "on_demand".to_string());
            (key, device_id, device_token, media_sync_policy)
        };

        loop {
            let cursor = {
                let _guard = with_db_access(&self.db).await;
                metadata::get_pull_cursor(&db).await?
            };
            let (envelopes, next_cursor, has_more) =
                pull::pull(&key, url, &device_id, &device_token, cursor.as_ref()).await?;

            let ctx = apply::ApplyCtx {
                base_url: url,
                key: &key,
                device_id: &device_id,
                device_token: &device_token,
                media_sync_policy: &media_sync_policy,
            };
            {
                let _guard = with_db_access(&self.db).await;
                apply_pulled_changes(&db, &envelopes, next_cursor.as_ref(), &ctx).await?;
            }
            if !has_more {
                break;
            }
        }

        let _ = push::push(&self.db, &key, url, &media_sync_policy).await?;
        Ok(())
    }

    pub async fn push_pending(&self) -> Result<()> {
        let (url, pass) = {
            let u = self.server_url.lock().unwrap().clone();
            let p = self.passphrase.lock().unwrap().clone();
            (u, p)
        };
        let Some(url) = url else {
            return Ok(());
        };
        let Some(pass) = pass else {
            return Ok(());
        };
        let db = get_database(&self.db);
        let (key, media_sync_policy) = {
            let _guard = with_db_access(&self.db).await;
            let key = metadata::verify_key(&db, &pass).await?;
            let media_sync_policy = settings::get_setting(db.clone(), "sync.media_sync_policy")
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| "on_demand".to_string());
            (key, media_sync_policy)
        };
        let _ = push::push(&self.db, &key, &url, &media_sync_policy).await?;
        Ok(())
    }

    pub async fn status(&self) -> Result<SyncStatus> {
        let _guard = with_db_access(&self.db).await;
        let db = get_database(&self.db);
        let conn = db.connect().map_err(AppError::LibSQL)?;

        let server_url = self.server_url.lock().unwrap().clone();
        let has_server_url = server_url.is_some();
        let has_device_token = metadata::get_device_token(db.as_ref()).await?.is_some();
        let connected = has_server_url && has_device_token;
        let needs_passphrase = connected && self.passphrase.lock().unwrap().is_none();

        let mut rows = conn
            .query("SELECT COUNT(*) FROM _sync_outbox", libsql::params![])
            .await
            .map_err(AppError::LibSQL)?;
        let pending: i64 = if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            row.get(0).unwrap_or(0)
        } else {
            0
        };

        Ok(SyncStatus {
            connected,
            pending_changes: pending as u32,
            last_sync: metadata::get_last_sync(&db).await.ok().flatten(),
            needs_passphrase,
            server_url,
        })
    }

    pub fn is_syncing(&self) -> bool {
        *self.is_syncing.lock().unwrap()
    }

    pub async fn reconnect(&self, passphrase: String) -> Result<()> {
        let url = self.server_url.lock().unwrap().clone();
        let Some(_) = url else {
            return Err(AppError::Sync("sync not configured".into()));
        };
        let _guard = with_db_access(&self.db).await;
        let db = get_database(&self.db);
        metadata::verify_key(&db, &passphrase).await?;
        *self.passphrase.lock().unwrap() = Some(passphrase);
        Ok(())
    }

    pub async fn disconnect(&self, app: &AppHandle) -> Result<()> {
        let _ = self.clear_passphrase(app);
        *self.server_url.lock().unwrap() = None;
        *self.passphrase.lock().unwrap() = None;

        let _guard = with_db_access(&self.db).await;
        let db = get_database(&self.db);
        metadata::clear_sync_configuration(&db).await?;
        Ok(())
    }

    pub fn try_get_url(&self) -> Option<String> {
        self.server_url.lock().unwrap().clone()
    }

    pub async fn try_get_key(&self) -> Option<[u8; 32]> {
        let pass = self.passphrase.lock().unwrap().clone()?;
        let _guard = with_db_access(&self.db).await;
        let db = get_database(&self.db);
        metadata::verify_key(db.as_ref(), &pass).await.ok()
    }

    pub async fn get_device_id(&self) -> Result<String> {
        let _guard = with_db_access(&self.db).await;
        let db = get_database(&self.db);
        metadata::get_device_id(&db).await
    }

    pub async fn get_device_hostname(&self) -> Result<String> {
        let _guard = with_db_access(&self.db).await;
        let db = get_database(&self.db);
        metadata::get_device_hostname(&db).await
    }

    pub async fn get_device_token(&self) -> Result<Option<String>> {
        let _guard = with_db_access(&self.db).await;
        let db = get_database(&self.db);
        metadata::get_device_token(&db).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::sync::types::{ChangeEnvelope, ChangeOp, PullCursor};
    use libsql::Builder;

    async fn test_db() -> libsql::Database {
        let path = std::env::temp_dir().join(format!(
            "aether-sync-engine-test-{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let db = Builder::new_local(path).build().await.unwrap();
        migrations::run_migrations(&db).await.unwrap();
        db
    }

    #[tokio::test]
    async fn sync_without_configuration_returns_disconnected_status() {
        let db = test_db().await;
        let state = crate::DbState {
            database: Arc::new(Mutex::new(Arc::new(db))),
            db_access: Arc::new(tokio::sync::Mutex::new(())),
            db_access_waiters: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        };
        let engine = SyncEngine::new(state);

        let status = engine.sync().await.unwrap();

        assert!(!status.connected);
        assert!(!status.needs_passphrase);
        assert_eq!(status.pending_changes, 0);
        assert_eq!(status.last_sync, None);
        assert_eq!(status.server_url, None);
    }

    #[tokio::test]
    async fn apply_pulled_changes_only_updates_cursor_after_success() {
        let db = test_db().await;
        let initial_cursor = PullCursor {
            received_at: 10,
            change_id: 1,
        };
        metadata::set_pull_cursor(&db, &initial_cursor)
            .await
            .unwrap();

        let ctx = apply::ApplyCtx {
            base_url: "https://sync.example.com",
            key: &[0; 32],
            device_id: "device-a",
            device_token: "token-a",
            media_sync_policy: "on_demand",
        };

        let bad_change = ChangeEnvelope {
            entity: "resource_links".into(),
            id: "link-1".into(),
            op: ChangeOp::Upsert,
            data: Some(serde_json::json!({
                "id": "link-1",
                "source_type": "entry",
                "source_id": "entry-1",
                "target_type": "task",
                "created_at": "2026-01-01T00:00:00Z"
            })),
            updated_at: 100,
            device_id: "device-a".into(),
            device_hostname: "host-a".into(),
        };
        let next_cursor = PullCursor {
            received_at: 20,
            change_id: 2,
        };

        let err = apply_pulled_changes(&db, &[bad_change], Some(&next_cursor), &ctx)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("failed to apply change"));
        assert_eq!(
            metadata::get_pull_cursor(&db).await.unwrap(),
            Some(initial_cursor)
        );
        assert_eq!(metadata::get_last_sync(&db).await.unwrap(), Some(10));
    }

    #[tokio::test]
    async fn applies_task_tags_after_their_task_and_tag() {
        let db = test_db().await;
        let ctx = apply::ApplyCtx {
            base_url: "https://sync.example.com",
            key: &[0; 32],
            device_id: "device-a",
            device_token: "token-a",
            media_sync_policy: "on_demand",
        };
        let changes = vec![
            ChangeEnvelope {
                entity: "task_tags".into(),
                id: "task-1|tag-1".into(),
                op: ChangeOp::Upsert,
                data: Some(serde_json::json!({ "task_id": "task-1", "tag_id": "tag-1" })),
                updated_at: 300,
                device_id: "device-a".into(),
                device_hostname: "host-a".into(),
            },
            ChangeEnvelope {
                entity: "tasks".into(),
                id: "task-1".into(),
                op: ChangeOp::Upsert,
                data: Some(serde_json::json!({ "id": "task-1", "title": "Task" })),
                updated_at: 200,
                device_id: "device-a".into(),
                device_hostname: "host-a".into(),
            },
            ChangeEnvelope {
                entity: "tags".into(),
                id: "tag-1".into(),
                op: ChangeOp::Upsert,
                data: Some(serde_json::json!({ "id": "tag-1", "name": "Tag" })),
                updated_at: 100,
                device_id: "device-a".into(),
                device_hostname: "host-a".into(),
            },
        ];

        apply_pulled_changes(&db, &changes, None, &ctx)
            .await
            .unwrap();

        let conn = db.connect().unwrap();
        let mut rows = conn
            .query(
                "SELECT task_id, tag_id FROM task_tags WHERE _sync_id = ?1",
                libsql::params!["task-1|tag-1"],
            )
            .await
            .unwrap();
        let row = rows.next().await.unwrap().unwrap();
        assert_eq!(row.get::<String>(0).unwrap(), "task-1");
        assert_eq!(row.get::<String>(1).unwrap(), "tag-1");
    }

    #[tokio::test]
    async fn defers_a_task_until_its_goal_arrives_in_a_later_pull_batch() {
        let db = test_db().await;
        let ctx = apply::ApplyCtx {
            base_url: "https://sync.example.com",
            key: &[0; 32],
            device_id: "device-a",
            device_token: "token-a",
            media_sync_policy: "on_demand",
        };
        let task = ChangeEnvelope {
            entity: "tasks".into(),
            id: "task-1".into(),
            op: ChangeOp::Upsert,
            data: Some(serde_json::json!({
                "id": "task-1",
                "title": "Task",
                "goal_id": "goal-1"
            })),
            updated_at: 100,
            device_id: "device-a".into(),
            device_hostname: "host-a".into(),
        };

        apply_pulled_changes(&db, &[task], None, &ctx)
            .await
            .unwrap();

        let conn = db.connect().unwrap();
        let mut rows = conn
            .query(
                "SELECT COUNT(*) FROM _sync_deferred_changes WHERE entity = 'tasks' AND entity_id = 'task-1'",
                libsql::params![],
            )
            .await
            .unwrap();
        assert_eq!(
            rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
            1
        );
        drop(rows);

        let goal = ChangeEnvelope {
            entity: "goals".into(),
            id: "goal-1".into(),
            op: ChangeOp::Upsert,
            data: Some(serde_json::json!({ "id": "goal-1", "name": "Goal" })),
            updated_at: 200,
            device_id: "device-a".into(),
            device_hostname: "host-a".into(),
        };

        apply_pulled_changes(&db, &[goal], None, &ctx)
            .await
            .unwrap();

        let mut rows = conn
            .query(
                "SELECT goal_id FROM tasks WHERE id = 'task-1'",
                libsql::params![],
            )
            .await
            .unwrap();
        assert_eq!(
            rows.next()
                .await
                .unwrap()
                .unwrap()
                .get::<String>(0)
                .unwrap(),
            "goal-1"
        );
        drop(rows);
        let mut rows = conn
            .query(
                "SELECT COUNT(*) FROM _sync_deferred_changes",
                libsql::params![],
            )
            .await
            .unwrap();
        assert_eq!(
            rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
            0
        );
    }
}
