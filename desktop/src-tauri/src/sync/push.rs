//! Push: read _sync_outbox, build ChangeEnvelopes, encrypt, POST /push, clear outbox on success.

use crate::error::{AppError, Result};
use crate::media::get_media_directory;
use crate::sync::encryption;
use crate::sync::media;
use crate::sync::metadata;
use crate::sync::types::{ChangeEnvelope, ChangeOp, EncryptedChange, PushRequest};
use libsql::Database;
use std::sync::Arc;

pub async fn push(
    db: Arc<Database>,
    key: &[u8; 32],
    base_url: &str,
    media_sync_policy: &str,
) -> Result<usize> {
    tracing::info!("[SYNC-PUSH] Starting push operation to {}", base_url);
    let device_id = metadata::get_device_id(&db).await?;
    tracing::debug!("[SYNC-PUSH] Device ID: {}", device_id);
    let (envelopes, to_delete) = read_outbox_and_build(&db).await?;
    if envelopes.is_empty() {
        tracing::info!("[SYNC-PUSH] No changes to push");
        return Ok(0);
    }
    tracing::info!("[SYNC-PUSH] Prepared {} changes to push", envelopes.len());

    // Upload media blobs when policy is "auto"
    if media_sync_policy == "auto" {
        tracing::debug!("[SYNC-PUSH] Media sync policy is 'auto', uploading media blobs");
        if let Ok(media_dir) = get_media_directory() {
            let mut uploaded = 0;
            for e in &envelopes {
                if e.entity != "media_items" || e.data.is_none() {
                    continue;
                }
                let data = e.data.as_ref().unwrap();
                let content_hash = data.get("content_hash").and_then(|v| v.as_str());
                let file_path = data.get("file_path").and_then(|v| v.as_str());
                if let (Some(hash), Some(fp)) = (content_hash, file_path) {
                    let full = media_dir.join(fp);
                    if let Ok(bytes) = std::fs::read(&full) {
                        if let Ok(enc) = encryption::encrypt_blob(key, &bytes) {
                            if media::upload_media(base_url, hash, &enc).await.is_ok() {
                                uploaded += 1;
                                tracing::debug!("[SYNC-PUSH] Uploaded media blob: {}", hash);
                            } else {
                                tracing::warn!("[SYNC-PUSH] Failed to upload media blob {}", hash);
                            }
                        }
                    }
                }
            }
            tracing::info!("[SYNC-PUSH] Uploaded {} media blobs", uploaded);
        }
    }

    tracing::info!("[SYNC-PUSH] Encrypting {} changes", envelopes.len());
    let mut encrypted = Vec::with_capacity(envelopes.len());
    for e in &envelopes {
        let json = serde_json::to_vec(e).map_err(AppError::Serialization)?;
        let (nonce, ct) = encryption::encrypt(key, &json)?;
        encrypted.push(EncryptedChange { nonce, ciphertext: ct });
    }

    let url = format!("{}/push", base_url.trim_end_matches('/'));
    tracing::info!("[SYNC-PUSH] Sending POST request to {}", url);
    let body = PushRequest {
        device_id,
        changes: encrypted,
    };
    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("[SYNC-PUSH] Network error: {}", e);
            AppError::Sync(format!("push request: {}", e))
        })?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        tracing::error!("[SYNC-PUSH] Server returned error {}: {}", status, text);
        return Err(AppError::Sync(format!("push failed {}: {}", status, text)));
    }

    tracing::info!("[SYNC-PUSH] Server accepted changes, deleting {} rows from outbox", to_delete.len());
    delete_outbox_rows(&db, &to_delete).await?;
    tracing::info!("[SYNC-PUSH] Push completed successfully");
    Ok(to_delete.len())
}

async fn read_outbox_and_build(
    db: &Database,
) -> Result<(Vec<ChangeEnvelope>, Vec<(String, String)>)> {
    tracing::debug!("[SYNC-PUSH] Reading outbox");
    // #region agent log
    let thread_id = format!("{:?}", std::thread::current().id());
    crate::db::connection::debug_log_connect("push.rs:100", &thread_id);
    // #endregion
    let conn = db.connect().map_err(|e| {
        // #region agent log
        crate::db::connection::debug_log_connect_error("push.rs:100", &e.to_string(), &thread_id);
        // #endregion
        AppError::LibSQL(e)
    })?;
    let mut rows = conn
        .query(
            "SELECT entity, entity_id, op, queued_at FROM _sync_outbox ORDER BY queued_at",
            libsql::params![],
        )
        .await
        .map_err(AppError::LibSQL)?;

    // Dedupe by (entity, entity_id): keep the latest (last) row.
    let mut seen: std::collections::HashMap<(String, String), (String, i64)> = std::collections::HashMap::new();
    let mut total_rows = 0;
    while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
        let entity: String = row.get(0).map_err(AppError::LibSQL)?;
        let entity_id: String = row.get(1).map_err(AppError::LibSQL)?;
        let op: String = row.get(2).map_err(AppError::LibSQL)?;
        let queued_at: i64 = row.get(3).map_err(AppError::LibSQL)?;
        seen.insert((entity.clone(), entity_id.clone()), (op, queued_at));
        total_rows += 1;
    }
    tracing::debug!("[SYNC-PUSH] Read {} rows from outbox, {} unique entities after deduplication", total_rows, seen.len());

    let mut envelopes = Vec::new();
    let mut to_delete = Vec::new();
    for ((entity, entity_id), (op, _queued_at)) in seen {
        let change_op = if op == "delete" {
            ChangeOp::Delete
        } else {
            ChangeOp::Upsert
        };

        let (data, updated_at) = if change_op == ChangeOp::Upsert {
            match fetch_row_json(&db, &entity, &entity_id).await? {
                Some((d, ts)) => (Some(d), ts),
                None => continue, // row already deleted, skip
            }
        } else {
            (None, metadata::get_last_sync(&db).await?.unwrap_or(0))
        };

        // For delete we need updated_at; use last_sync or fetch _updated_at.
        let updated_at = if change_op == ChangeOp::Delete {
            fetch_updated_at(&db, &entity, &entity_id).await?.unwrap_or(updated_at)
        } else {
            data.as_ref().and_then(|d| d.get("_updated_at").and_then(|v| v.as_i64())).unwrap_or(updated_at)
        };

        let envelope = ChangeEnvelope {
            entity: entity.clone(),
            id: entity_id.clone(),
            op: change_op,
            data,
            updated_at,
        };
        envelopes.push(envelope);
        to_delete.push((entity, entity_id));
    }

    Ok((envelopes, to_delete))
}

async fn fetch_row_json(
    db: &Database,
    entity: &str,
    entity_id: &str,
) -> Result<Option<(serde_json::Value, i64)>> {
    match entity {
        "entries" => fetch_entries_row(db, entity_id).await,
        "tags" => fetch_tags_row(db, entity_id).await,
        "tasks" => fetch_tasks_row(db, entity_id).await,
        "goals" => fetch_goals_row(db, entity_id).await,
        "canvases" => fetch_canvases_row(db, entity_id).await,
        "bookmarks" => fetch_bookmarks_row(db, entity_id).await,
        "media_items" => fetch_media_items_row(db, entity_id).await,
        "audio_transcriptions" => fetch_audio_transcriptions_row(db, entity_id).await,
        "entry_tags" => fetch_entry_tags_row(db, entity_id).await,
        "task_tags" => fetch_task_tags_row(db, entity_id).await,
        "goal_tags" => fetch_goal_tags_row(db, entity_id).await,
        "goal_instance_tags" => fetch_goal_instance_tags_row(db, entity_id).await,
        "bookmark_tags" => fetch_bookmark_tags_row(db, entity_id).await,
        "goal_instances" => fetch_goal_instances_row(db, entity_id).await,
        "subtasks" => fetch_subtasks_row(db, entity_id).await,
        _ => Ok(None),
    }
}

async fn fetch_entry_tags_row(
    db: &Database,
    entity_id: &str,
) -> Result<Option<(serde_json::Value, i64)>> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT entry_id, tag_id, _sync_id, _updated_at, _deleted, _extra FROM entry_tags WHERE _sync_id = ?1",
            libsql::params![entity_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
    let row = match rows.next().await.map_err(AppError::LibSQL)? {
        Some(r) => r,
        None => return Ok(None),
    };
    let ts = row.get::<Option<i64>>(3).ok().flatten().unwrap_or(0);
    let data = serde_json::json!({
        "entry_id": row.get::<String>(0).ok(),
        "tag_id": row.get::<String>(1).ok(),
        "_sync_id": row.get::<Option<String>>(2).ok().flatten(),
        "_updated_at": row.get::<Option<i64>>(3).ok().flatten(),
        "_deleted": row.get::<i64>(4).map(|v| v != 0).unwrap_or(false),
        "_extra": row.get::<Option<String>>(5).ok().flatten().unwrap_or_else(|| "{}".into()),
    });
    Ok(Some((data, ts)))
}

async fn fetch_task_tags_row(
    db: &Database,
    entity_id: &str,
) -> Result<Option<(serde_json::Value, i64)>> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT task_id, tag_id, _sync_id, _updated_at, _deleted, _extra FROM task_tags WHERE _sync_id = ?1",
            libsql::params![entity_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
    let row = match rows.next().await.map_err(AppError::LibSQL)? {
        Some(r) => r,
        None => return Ok(None),
    };
    let ts = row.get::<Option<i64>>(3).ok().flatten().unwrap_or(0);
    let data = serde_json::json!({
        "task_id": row.get::<String>(0).ok(),
        "tag_id": row.get::<String>(1).ok(),
        "_sync_id": row.get::<Option<String>>(2).ok().flatten(),
        "_updated_at": row.get::<Option<i64>>(3).ok().flatten(),
        "_deleted": row.get::<i64>(4).map(|v| v != 0).unwrap_or(false),
        "_extra": row.get::<Option<String>>(5).ok().flatten().unwrap_or_else(|| "{}".into()),
    });
    Ok(Some((data, ts)))
}

async fn fetch_goal_tags_row(
    db: &Database,
    entity_id: &str,
) -> Result<Option<(serde_json::Value, i64)>> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT goal_id, tag_id, _sync_id, _updated_at, _deleted, _extra FROM goal_tags WHERE _sync_id = ?1",
            libsql::params![entity_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
    let row = match rows.next().await.map_err(AppError::LibSQL)? {
        Some(r) => r,
        None => return Ok(None),
    };
    let ts = row.get::<Option<i64>>(3).ok().flatten().unwrap_or(0);
    let data = serde_json::json!({
        "goal_id": row.get::<String>(0).ok(),
        "tag_id": row.get::<String>(1).ok(),
        "_sync_id": row.get::<Option<String>>(2).ok().flatten(),
        "_updated_at": row.get::<Option<i64>>(3).ok().flatten(),
        "_deleted": row.get::<i64>(4).map(|v| v != 0).unwrap_or(false),
        "_extra": row.get::<Option<String>>(5).ok().flatten().unwrap_or_else(|| "{}".into()),
    });
    Ok(Some((data, ts)))
}

async fn fetch_goal_instance_tags_row(
    db: &Database,
    entity_id: &str,
) -> Result<Option<(serde_json::Value, i64)>> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT goal_instance_id, tag_id, _sync_id, _updated_at, _deleted, _extra FROM goal_instance_tags WHERE _sync_id = ?1",
            libsql::params![entity_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
    let row = match rows.next().await.map_err(AppError::LibSQL)? {
        Some(r) => r,
        None => return Ok(None),
    };
    let ts = row.get::<Option<i64>>(3).ok().flatten().unwrap_or(0);
    let data = serde_json::json!({
        "goal_instance_id": row.get::<String>(0).ok(),
        "tag_id": row.get::<String>(1).ok(),
        "_sync_id": row.get::<Option<String>>(2).ok().flatten(),
        "_updated_at": row.get::<Option<i64>>(3).ok().flatten(),
        "_deleted": row.get::<i64>(4).map(|v| v != 0).unwrap_or(false),
        "_extra": row.get::<Option<String>>(5).ok().flatten().unwrap_or_else(|| "{}".into()),
    });
    Ok(Some((data, ts)))
}

async fn fetch_bookmark_tags_row(
    db: &Database,
    entity_id: &str,
) -> Result<Option<(serde_json::Value, i64)>> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT bookmark_id, tag_id, _sync_id, _updated_at, _deleted, _extra FROM bookmark_tags WHERE _sync_id = ?1",
            libsql::params![entity_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
    let row = match rows.next().await.map_err(AppError::LibSQL)? {
        Some(r) => r,
        None => return Ok(None),
    };
    let ts = row.get::<Option<i64>>(3).ok().flatten().unwrap_or(0);
    let data = serde_json::json!({
        "bookmark_id": row.get::<String>(0).ok(),
        "tag_id": row.get::<String>(1).ok(),
        "_sync_id": row.get::<Option<String>>(2).ok().flatten(),
        "_updated_at": row.get::<Option<i64>>(3).ok().flatten(),
        "_deleted": row.get::<i64>(4).map(|v| v != 0).unwrap_or(false),
        "_extra": row.get::<Option<String>>(5).ok().flatten().unwrap_or_else(|| "{}".into()),
    });
    Ok(Some((data, ts)))
}

async fn fetch_goal_instances_row(
    db: &Database,
    entity_id: &str,
) -> Result<Option<(serde_json::Value, i64)>> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT id, goal_id, period_start, period_end, status, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra FROM goal_instances WHERE _sync_id = ?1",
            libsql::params![entity_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
    let row = match rows.next().await.map_err(AppError::LibSQL)? {
        Some(r) => r,
        None => return Ok(None),
    };
    let ts = row.get::<Option<i64>>(9).ok().flatten().unwrap_or(0);
    let data = serde_json::json!({
        "id": row.get::<String>(0).ok(),
        "goal_id": row.get::<String>(1).ok(),
        "period_start": row.get::<String>(2).ok(),
        "period_end": row.get::<Option<String>>(3).ok().flatten(),
        "status": row.get::<String>(4).ok(),
        "created_at": row.get::<String>(5).ok(),
        "updated_at": row.get::<Option<String>>(6).ok().flatten(),
        "deleted_at": row.get::<Option<String>>(7).ok().flatten(),
        "_sync_id": row.get::<Option<String>>(8).ok().flatten(),
        "_updated_at": row.get::<Option<i64>>(9).ok().flatten(),
        "_deleted": row.get::<i64>(10).map(|v| v != 0).unwrap_or(false),
        "_extra": row.get::<Option<String>>(11).ok().flatten().unwrap_or_else(|| "{}".into()),
    });
    Ok(Some((data, ts)))
}

async fn fetch_subtasks_row(
    db: &Database,
    entity_id: &str,
) -> Result<Option<(serde_json::Value, i64)>> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT id, title, is_completed, task_id, order_index, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra FROM subtasks WHERE _sync_id = ?1",
            libsql::params![entity_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
    let row = match rows.next().await.map_err(AppError::LibSQL)? {
        Some(r) => r,
        None => return Ok(None),
    };
    let ts = row.get::<Option<i64>>(9).ok().flatten().unwrap_or(0);
    let data = serde_json::json!({
        "id": row.get::<String>(0).ok(),
        "title": row.get::<String>(1).ok(),
        "is_completed": row.get::<i64>(2).map(|v| v != 0).unwrap_or(false),
        "task_id": row.get::<String>(3).ok(),
        "order_index": row.get::<i64>(4).unwrap_or(0),
        "created_at": row.get::<String>(5).ok(),
        "updated_at": row.get::<String>(6).ok(),
        "deleted_at": row.get::<Option<String>>(7).ok().flatten(),
        "_sync_id": row.get::<Option<String>>(8).ok().flatten(),
        "_updated_at": row.get::<Option<i64>>(9).ok().flatten(),
        "_deleted": row.get::<i64>(10).map(|v| v != 0).unwrap_or(false),
        "_extra": row.get::<Option<String>>(11).ok().flatten().unwrap_or_else(|| "{}".into()),
    });
    Ok(Some((data, ts)))
}

async fn fetch_entries_row(
    db: &Database,
    entity_id: &str,
) -> Result<Option<(serde_json::Value, i64)>> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT id, document, created_at, is_pinned, is_archived, is_deleted, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra FROM entries WHERE _sync_id = ?1",
            libsql::params![entity_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
    let row = match rows.next().await.map_err(AppError::LibSQL)? {
        Some(r) => r,
        None => return Ok(None),
    };
    let id: String = row.get(0).map_err(AppError::LibSQL)?;
    let document: String = row.get(1).map_err(AppError::LibSQL)?;
    let created_at: String = row.get(2).map_err(AppError::LibSQL)?;
    let is_pinned: i64 = row.get(3).map_err(AppError::LibSQL)?;
    let is_archived: i64 = row.get(4).map_err(AppError::LibSQL)?;
    let is_deleted: i64 = row.get(5).map_err(AppError::LibSQL)?;
    let updated_at: String = row.get(6).map_err(AppError::LibSQL)?;
    let deleted_at: Option<String> = row.get(7).map_err(AppError::LibSQL)?;
    let _sync_id: Option<String> = row.get(8).ok();
    let _updated_at: Option<i64> = row.get(9).ok();
    let _deleted: i64 = row.get(10).unwrap_or(0);
    let _extra: Option<String> = row.get(11).ok().flatten();

    let ts = _updated_at.unwrap_or(0);
    let data = serde_json::json!({
        "id": id,
        "document": document,
        "created_at": created_at,
        "is_pinned": is_pinned != 0,
        "is_archived": is_archived != 0,
        "is_deleted": is_deleted != 0,
        "updated_at": updated_at,
        "deleted_at": deleted_at,
        "_sync_id": _sync_id,
        "_updated_at": _updated_at,
        "_deleted": _deleted != 0,
        "_extra": _extra.unwrap_or_else(|| "{}".into()),
    });
    Ok(Some((data, ts)))
}

async fn fetch_tags_row(
    db: &Database,
    entity_id: &str,
) -> Result<Option<(serde_json::Value, i64)>> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT id, name, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra FROM tags WHERE _sync_id = ?1",
            libsql::params![entity_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
    let row = match rows.next().await.map_err(AppError::LibSQL)? {
        Some(r) => r,
        None => return Ok(None),
    };
    let id: String = row.get(0).map_err(AppError::LibSQL)?;
    let name: String = row.get(1).map_err(AppError::LibSQL)?;
    let created_at: String = row.get(2).map_err(AppError::LibSQL)?;
    let updated_at: String = row.get(3).map_err(AppError::LibSQL)?;
    let deleted_at: Option<String> = row.get(4).map_err(AppError::LibSQL)?;
    let _sync_id: Option<String> = row.get(5).ok();
    let _updated_at: Option<i64> = row.get(6).ok();
    let _deleted: i64 = row.get(7).unwrap_or(0);
    let _extra: Option<String> = row.get(8).ok().flatten();

    let ts = _updated_at.unwrap_or(0);
    let data = serde_json::json!({
        "id": id,
        "name": name,
        "created_at": created_at,
        "updated_at": updated_at,
        "deleted_at": deleted_at,
        "_sync_id": _sync_id,
        "_updated_at": _updated_at,
        "_deleted": _deleted != 0,
        "_extra": _extra.unwrap_or_else(|| "{}".into()),
    });
    Ok(Some((data, ts)))
}

async fn fetch_tasks_row(
    db: &Database,
    entity_id: &str,
) -> Result<Option<(serde_json::Value, i64)>> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT id, title, description, is_completed, due_date, goal_instance_id, goal_id, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra FROM tasks WHERE _sync_id = ?1",
            libsql::params![entity_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
    let row = match rows.next().await.map_err(AppError::LibSQL)? {
        Some(r) => r,
        None => return Ok(None),
    };
    let ts = row.get::<Option<i64>>(11).ok().flatten().unwrap_or(0);
    let data = serde_json::json!({
        "id": row.get::<String>(0).ok(),
        "title": row.get::<String>(1).ok(),
        "description": row.get::<Option<String>>(2).ok().flatten(),
        "is_completed": row.get::<i64>(3).map(|v| v != 0).unwrap_or(false),
        "due_date": row.get::<Option<String>>(4).ok().flatten(),
        "goal_instance_id": row.get::<Option<String>>(5).ok().flatten(),
        "goal_id": row.get::<Option<String>>(6).ok().flatten(),
        "created_at": row.get::<String>(7).ok(),
        "updated_at": row.get::<String>(8).ok(),
        "deleted_at": row.get::<Option<String>>(9).ok().flatten(),
        "_sync_id": row.get::<Option<String>>(10).ok().flatten(),
        "_updated_at": row.get::<Option<i64>>(11).ok().flatten(),
        "_deleted": row.get::<i64>(12).map(|v| v != 0).unwrap_or(false),
        "_extra": row.get::<Option<String>>(13).ok().flatten().unwrap_or_else(|| "{}".into()),
    });
    Ok(Some((data, ts)))
}

async fn fetch_goals_row(
    db: &Database,
    entity_id: &str,
) -> Result<Option<(serde_json::Value, i64)>> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT id, name, description, is_non_recurring, recurrence_type, recurrence_interval, recurrence_anchor, recurrence_meta, timezone, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra FROM goals WHERE _sync_id = ?1",
            libsql::params![entity_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
    let row = match rows.next().await.map_err(AppError::LibSQL)? {
        Some(r) => r,
        None => return Ok(None),
    };
    let ts = row.get::<Option<i64>>(13).ok().flatten().unwrap_or(0);
    let data = serde_json::json!({
        "id": row.get::<String>(0).ok(),
        "name": row.get::<String>(1).ok(),
        "description": row.get::<Option<String>>(2).ok().flatten(),
        "is_non_recurring": row.get::<i64>(3).map(|v| v != 0).unwrap_or(true),
        "recurrence_type": row.get::<Option<String>>(4).ok().flatten(),
        "recurrence_interval": row.get::<Option<i64>>(5).ok().flatten(),
        "recurrence_anchor": row.get::<Option<String>>(6).ok().flatten(),
        "recurrence_meta": row.get::<Option<String>>(7).ok().flatten(),
        "timezone": row.get::<String>(8).ok(),
        "created_at": row.get::<String>(9).ok(),
        "updated_at": row.get::<String>(10).ok(),
        "deleted_at": row.get::<Option<String>>(11).ok().flatten(),
        "_sync_id": row.get::<Option<String>>(12).ok().flatten(),
        "_updated_at": row.get::<Option<i64>>(13).ok().flatten(),
        "_deleted": row.get::<i64>(14).map(|v| v != 0).unwrap_or(false),
        "_extra": row.get::<Option<String>>(15).ok().flatten().unwrap_or_else(|| "{}".into()),
    });
    Ok(Some((data, ts)))
}

async fn fetch_canvases_row(
    db: &Database,
    entity_id: &str,
) -> Result<Option<(serde_json::Value, i64)>> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT id, name, canvas_data, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra FROM canvases WHERE _sync_id = ?1",
            libsql::params![entity_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
    let row = match rows.next().await.map_err(AppError::LibSQL)? {
        Some(r) => r,
        None => return Ok(None),
    };
    let ts = row.get::<Option<i64>>(7).ok().flatten().unwrap_or(0);
    let data = serde_json::json!({
        "id": row.get::<String>(0).ok(),
        "name": row.get::<String>(1).ok(),
        "canvas_data": row.get::<String>(2).ok(),
        "created_at": row.get::<String>(3).ok(),
        "updated_at": row.get::<String>(4).ok(),
        "deleted_at": row.get::<Option<String>>(5).ok().flatten(),
        "_sync_id": row.get::<Option<String>>(6).ok().flatten(),
        "_updated_at": row.get::<Option<i64>>(7).ok().flatten(),
        "_deleted": row.get::<i64>(8).map(|v| v != 0).unwrap_or(false),
        "_extra": row.get::<Option<String>>(9).ok().flatten().unwrap_or_else(|| "{}".into()),
    });
    Ok(Some((data, ts)))
}

async fn fetch_bookmarks_row(
    db: &Database,
    entity_id: &str,
) -> Result<Option<(serde_json::Value, i64)>> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT id, url, title, description, image_url, favicon_url, site_name, author, published_at, content_type, metadata_json, is_archived, is_deleted, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra FROM bookmarks WHERE _sync_id = ?1",
            libsql::params![entity_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
    let row = match rows.next().await.map_err(AppError::LibSQL)? {
        Some(r) => r,
        None => return Ok(None),
    };
    let ts = row.get::<Option<i64>>(17).ok().flatten().unwrap_or(0);
    let data = serde_json::json!({
        "id": row.get::<String>(0).ok(),
        "url": row.get::<String>(1).ok(),
        "title": row.get::<Option<String>>(2).ok().flatten(),
        "description": row.get::<Option<String>>(3).ok().flatten(),
        "image_url": row.get::<Option<String>>(4).ok().flatten(),
        "favicon_url": row.get::<Option<String>>(5).ok().flatten(),
        "site_name": row.get::<Option<String>>(6).ok().flatten(),
        "author": row.get::<Option<String>>(7).ok().flatten(),
        "published_at": row.get::<Option<String>>(8).ok().flatten(),
        "content_type": row.get::<Option<String>>(9).ok().flatten(),
        "metadata_json": row.get::<Option<String>>(10).ok().flatten(),
        "is_archived": row.get::<i64>(11).map(|v| v != 0).unwrap_or(false),
        "is_deleted": row.get::<i64>(12).map(|v| v != 0).unwrap_or(false),
        "created_at": row.get::<String>(13).ok(),
        "updated_at": row.get::<String>(14).ok(),
        "deleted_at": row.get::<Option<String>>(15).ok().flatten(),
        "_sync_id": row.get::<Option<String>>(16).ok().flatten(),
        "_updated_at": row.get::<Option<i64>>(17).ok().flatten(),
        "_deleted": row.get::<i64>(18).map(|v| v != 0).unwrap_or(false),
        "_extra": row.get::<Option<String>>(19).ok().flatten().unwrap_or_else(|| "{}".into()),
    });
    Ok(Some((data, ts)))
}

async fn fetch_media_items_row(
    db: &Database,
    entity_id: &str,
) -> Result<Option<(serde_json::Value, i64)>> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT id, entity_type, entity_id, media_type, file_path, metadata, created_at, updated_at, _sync_id, _updated_at, _deleted, _extra FROM media_items WHERE _sync_id = ?1",
            libsql::params![entity_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
    let row = match rows.next().await.map_err(AppError::LibSQL)? {
        Some(r) => r,
        None => return Ok(None),
    };
    let ts = row.get::<Option<i64>>(9).ok().flatten().unwrap_or(0);
    let file_path: Option<String> = row.get(4).ok().flatten();
    let mut data = serde_json::json!({
        "id": row.get::<String>(0).ok(),
        "entity_type": row.get::<String>(1).ok(),
        "entity_id": row.get::<String>(2).ok(),
        "media_type": row.get::<String>(3).ok(),
        "file_path": row.get::<String>(4).ok(),
        "metadata": row.get::<String>(5).ok(),
        "created_at": row.get::<String>(6).ok(),
        "updated_at": row.get::<String>(7).ok(),
        "_sync_id": row.get::<Option<String>>(8).ok().flatten(),
        "_updated_at": row.get::<Option<i64>>(9).ok().flatten(),
        "_deleted": row.get::<i64>(10).map(|v| v != 0).unwrap_or(false),
        "_extra": row.get::<Option<String>>(11).ok().flatten().unwrap_or_else(|| "{}".into()),
    });
    if let Some(fp) = file_path {
        if let Ok(media_dir) = get_media_directory() {
            let full = media_dir.join(&fp);
            if let Ok(bytes) = std::fs::read(&full) {
                if let Some(obj) = data.as_object_mut() {
                    obj.insert(
                        "content_hash".into(),
                        serde_json::Value::String(media::media_hash(&bytes)),
                    );
                }
            }
        }
    }
    Ok(Some((data, ts)))
}

async fn fetch_audio_transcriptions_row(
    db: &Database,
    entity_id: &str,
) -> Result<Option<(serde_json::Value, i64)>> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT id, media_id, transcription_text, provider, provider_config, confidence_score, status, error_message, is_active, created_at, _sync_id, _updated_at, _deleted, _extra FROM audio_transcriptions WHERE _sync_id = ?1",
            libsql::params![entity_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
    let row = match rows.next().await.map_err(AppError::LibSQL)? {
        Some(r) => r,
        None => return Ok(None),
    };
    let ts = row.get::<Option<i64>>(11).ok().flatten().unwrap_or(0);
    let data = serde_json::json!({
        "id": row.get::<String>(0).ok(),
        "media_id": row.get::<String>(1).ok(),
        "transcription_text": row.get::<String>(2).ok(),
        "provider": row.get::<String>(3).ok(),
        "provider_config": row.get::<Option<String>>(4).ok().flatten(),
        "confidence_score": row.get::<Option<f64>>(5).ok().flatten(),
        "status": row.get::<String>(6).ok(),
        "error_message": row.get::<Option<String>>(7).ok().flatten(),
        "is_active": row.get::<i64>(8).map(|v| v != 0).unwrap_or(false),
        "created_at": row.get::<String>(9).ok(),
        "_sync_id": row.get::<Option<String>>(10).ok().flatten(),
        "_updated_at": row.get::<Option<i64>>(11).ok().flatten(),
        "_deleted": row.get::<i64>(12).map(|v| v != 0).unwrap_or(false),
        "_extra": row.get::<Option<String>>(13).ok().flatten().unwrap_or_else(|| "{}".into()),
    });
    Ok(Some((data, ts)))
}

async fn fetch_updated_at(db: &Database, entity: &str, entity_id: &str) -> Result<Option<i64>> {
    let table = match entity {
        "entries" => "entries",
        "tags" => "tags",
        "entry_tags" => "entry_tags",
        "goals" => "goals",
        "goal_tags" => "goal_tags",
        "goal_instances" => "goal_instances",
        "goal_instance_tags" => "goal_instance_tags",
        "tasks" => "tasks",
        "task_tags" => "task_tags",
        "subtasks" => "subtasks",
        "media_items" => "media_items",
        "audio_transcriptions" => "audio_transcriptions",
        "canvases" => "canvases",
        "bookmarks" => "bookmarks",
        "bookmark_tags" => "bookmark_tags",
        _ => return Ok(None),
    };
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let sql = format!("SELECT _updated_at FROM {} WHERE _sync_id = ?1", table);
    let mut rows = conn
        .query(&sql, libsql::params![entity_id])
        .await
        .map_err(AppError::LibSQL)?;
    if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
        let v: Option<i64> = row.get(0).ok();
        Ok(v)
    } else {
        Ok(None)
    }
}

async fn delete_outbox_rows(
    db: &Database,
    rows: &[(String, String)],
) -> Result<()> {
    if rows.is_empty() {
        return Ok(());
    }
    // #region agent log
    let thread_id = format!("{:?}", std::thread::current().id());
    crate::db::connection::debug_log_connect("push.rs:754", &thread_id);
    // #endregion
    let conn = db.connect().map_err(|e| {
        // #region agent log
        crate::db::connection::debug_log_connect_error("push.rs:754", &e.to_string(), &thread_id);
        // #endregion
        AppError::LibSQL(e)
    })?;
    for (entity, entity_id) in rows {
        conn.execute(
            "DELETE FROM _sync_outbox WHERE entity = ?1 AND entity_id = ?2",
            libsql::params![entity.as_str(), entity_id.as_str()],
        )
        .await
        .map_err(AppError::LibSQL)?;
    }
    Ok(())
}
