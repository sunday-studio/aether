//! Push: read _sync_outbox, build ChangeEnvelopes, encrypt, POST /push, clear outbox on success.

use crate::error::{AppError, Result};
use crate::sync::encryption;
use crate::sync::metadata;
use crate::sync::types::{ChangeEnvelope, ChangeOp, EncryptedChange, PushRequest};
use libsql::Database;
use std::sync::Arc;

pub async fn push(db: Arc<Database>, key: &[u8; 32], base_url: &str) -> Result<usize> {
    let device_id = metadata::get_device_id(&db).await?;
    let (envelopes, to_delete) = read_outbox_and_build(&db).await?;
    if envelopes.is_empty() {
        return Ok(0);
    }

    let mut encrypted = Vec::with_capacity(envelopes.len());
    for e in &envelopes {
        let json = serde_json::to_vec(e).map_err(AppError::Serialization)?;
        let (nonce, ct) = encryption::encrypt(key, &json)?;
        encrypted.push(EncryptedChange { nonce, ciphertext: ct });
    }

    let url = format!("{}/push", base_url.trim_end_matches('/'));
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
        .map_err(|e| AppError::Sync(format!("push request: {}", e)))?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(AppError::Sync(format!("push failed {}: {}", status, text)));
    }

    delete_outbox_rows(&db, &to_delete).await?;
    Ok(to_delete.len())
}

async fn read_outbox_and_build(
    db: &Database,
) -> Result<(Vec<ChangeEnvelope>, Vec<(String, String)>)> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT entity, entity_id, op, queued_at FROM _sync_outbox ORDER BY queued_at",
            libsql::params![],
        )
        .await
        .map_err(AppError::LibSQL)?;

    // Dedupe by (entity, entity_id): keep the latest (last) row.
    let mut seen: std::collections::HashMap<(String, String), (String, i64)> = std::collections::HashMap::new();
    while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
        let entity: String = row.get(0).map_err(AppError::LibSQL)?;
        let entity_id: String = row.get(1).map_err(AppError::LibSQL)?;
        let op: String = row.get(2).map_err(AppError::LibSQL)?;
        let queued_at: i64 = row.get(3).map_err(AppError::LibSQL)?;
        seen.insert((entity.clone(), entity_id.clone()), (op, queued_at));
    }

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
        _ => Ok(None),
    }
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
    let conn = db.connect().map_err(AppError::LibSQL)?;
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
