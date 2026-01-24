//! Last-write-wins apply: set _suppress_triggers, compare _updated_at, insert/update/soft-delete.

use crate::error::{AppError, Result};
use crate::sync::metadata;
use crate::sync::types::{ChangeEnvelope, ChangeOp};
use libsql::Database;

/// Apply one decrypted change with LWW. Caller must set _suppress_triggers before batch.
pub async fn apply_change(db: &Database, change: &ChangeEnvelope) -> Result<()> {
    let local_ts = get_local_updated_at(db, &change.entity, &change.id).await?;

    // LWW: if local is newer or equal, skip
    if let Some(ts) = local_ts {
        if change.updated_at <= ts {
            return Ok(());
        }
    }

    match &change.op {
        ChangeOp::Upsert => {
            if let Some(data) = &change.data {
                apply_upsert(db, &change.entity, &change.id, change.updated_at, data).await?;
            }
        }
        ChangeOp::Delete => {
            apply_soft_delete(db, &change.entity, &change.id, change.updated_at).await?;
        }
    }
    Ok(())
}

/// Set _suppress_triggers to avoid re-queuing applied changes. Clear to '0' when done.
pub async fn with_suppress_triggers<F, R>(db: &Database, f: F) -> Result<R>
where
    F: std::future::Future<Output = Result<R>>,
{
    metadata::set_suppress_triggers(db, "1").await?;
    let res = f.await;
    let _ = metadata::set_suppress_triggers(db, "0").await;
    res
}

async fn get_local_updated_at(
    db: &Database,
    entity: &str,
    entity_id: &str,
) -> Result<Option<i64>> {
    let Some(table) = entity_to_table(entity) else {
        return Ok(None);
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

fn entity_to_table(entity: &str) -> Option<&'static str> {
    Some(match entity {
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
        _ => return None,
    })
}

async fn apply_upsert(
    db: &Database,
    entity: &str,
    entity_id: &str,
    updated_at: i64,
    data: &serde_json::Value,
) -> Result<()> {
    let _ = data
        .as_object()
        .ok_or_else(|| AppError::Sync("upsert data must be a JSON object".into()))?;

    match entity {
        "entries" => apply_entries_upsert(db, entity_id, updated_at, data).await,
        "tags" => apply_tags_upsert(db, entity_id, updated_at, data).await,
        _ => store_unknown(db, entity, entity_id, updated_at, data).await,
    }
}

async fn apply_entries_upsert(
    db: &Database,
    entity_id: &str,
    updated_at: i64,
    data: &serde_json::Value,
) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let obj = data.as_object().ok_or_else(|| AppError::Sync("entries: object expected".into()))?;
    let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or(entity_id);
    let document = obj.get("document").and_then(|v| v.as_str()).unwrap_or("");
    let created_at = obj.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    let is_pinned = obj.get("is_pinned").and_then(|v| v.as_bool()).unwrap_or(false);
    let is_archived = obj.get("is_archived").and_then(|v| v.as_bool()).unwrap_or(false);
    let is_deleted = obj.get("is_deleted").and_then(|v| v.as_bool()).unwrap_or(false);
    let updated_at_s = obj.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
    let deleted_at = obj.get("deleted_at").and_then(|v| v.as_str());
    let extra = obj.get("_extra").and_then(|v| v.as_str()).unwrap_or("{}");

    conn.execute(
        "INSERT OR REPLACE INTO entries (id, document, created_at, is_pinned, is_archived, is_deleted, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, ?11)",
        libsql::params![
            id,
            document,
            created_at,
            if is_pinned { 1i64 } else { 0 },
            if is_archived { 1i64 } else { 0 },
            if is_deleted { 1i64 } else { 0 },
            updated_at_s,
            deleted_at,
            entity_id,
            updated_at,
            extra,
        ],
    )
    .await
    .map_err(AppError::LibSQL)?;
    Ok(())
}

async fn apply_tags_upsert(
    db: &Database,
    entity_id: &str,
    updated_at: i64,
    data: &serde_json::Value,
) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let obj = data.as_object().ok_or_else(|| AppError::Sync("tags: object expected".into()))?;
    let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or(entity_id);
    let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let created_at = obj.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    let updated_at_s = obj.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
    let deleted_at = obj.get("deleted_at").and_then(|v| v.as_str());
    let extra = obj.get("_extra").and_then(|v| v.as_str()).unwrap_or("{}");

    conn.execute(
        "INSERT OR REPLACE INTO tags (id, name, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8)",
        libsql::params![
            id,
            name,
            created_at,
            updated_at_s,
            deleted_at,
            entity_id,
            updated_at,
            extra,
        ],
    )
    .await
    .map_err(AppError::LibSQL)?;
    Ok(())
}

async fn store_unknown(
    db: &Database,
    entity: &str,
    entity_id: &str,
    updated_at: i64,
    data: &serde_json::Value,
) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let data_str = serde_json::to_string(data).map_err(AppError::Serialization)?;
    conn.execute(
        "INSERT OR REPLACE INTO _sync_unknown (entity, entity_id, data, updated_at) VALUES (?1, ?2, ?3, ?4)",
        libsql::params![entity, entity_id, data_str, updated_at],
    )
    .await
    .map_err(AppError::LibSQL)?;
    Ok(())
}

async fn apply_soft_delete(db: &Database, entity: &str, entity_id: &str, updated_at: i64) -> Result<()> {
    let Some(table) = entity_to_table(entity) else { return Ok(()); };
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let sql = format!(
        "UPDATE {} SET _deleted = 1, _updated_at = ?1 WHERE _sync_id = ?2",
        table
    );
    conn.execute(&sql, libsql::params![updated_at, entity_id])
        .await
        .map_err(AppError::LibSQL)?;
    Ok(())
}
