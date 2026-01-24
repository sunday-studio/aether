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
        "tasks" => apply_tasks_upsert(db, entity_id, updated_at, data).await,
        "goals" => apply_goals_upsert(db, entity_id, updated_at, data).await,
        "canvases" => apply_canvases_upsert(db, entity_id, updated_at, data).await,
        "bookmarks" => apply_bookmarks_upsert(db, entity_id, updated_at, data).await,
        "media_items" => apply_media_items_upsert(db, entity_id, updated_at, data).await,
        "audio_transcriptions" => apply_audio_transcriptions_upsert(db, entity_id, updated_at, data).await,
        "entry_tags" => apply_entry_tags_upsert(db, entity_id, updated_at, data).await,
        "task_tags" => apply_task_tags_upsert(db, entity_id, updated_at, data).await,
        "goal_tags" => apply_goal_tags_upsert(db, entity_id, updated_at, data).await,
        "goal_instance_tags" => apply_goal_instance_tags_upsert(db, entity_id, updated_at, data).await,
        "bookmark_tags" => apply_bookmark_tags_upsert(db, entity_id, updated_at, data).await,
        "goal_instances" => apply_goal_instances_upsert(db, entity_id, updated_at, data).await,
        "subtasks" => apply_subtasks_upsert(db, entity_id, updated_at, data).await,
        _ => store_unknown(db, entity, entity_id, updated_at, data).await,
    }
}

async fn apply_entry_tags_upsert(
    db: &Database,
    entity_id: &str,
    updated_at: i64,
    data: &serde_json::Value,
) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let obj = data.as_object().ok_or_else(|| AppError::Sync("entry_tags: object expected".into()))?;
    let entry_id = obj.get("entry_id").and_then(|v| v.as_str()).unwrap_or_else(|| entity_id.split('|').next().unwrap_or(entity_id));
    let tag_id = obj.get("tag_id").and_then(|v| v.as_str()).unwrap_or_else(|| entity_id.split('|').nth(1).unwrap_or(""));
    let extra = obj.get("_extra").and_then(|v| v.as_str()).unwrap_or("{}");
    conn.execute(
        "INSERT OR REPLACE INTO entry_tags (entry_id, tag_id, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
        libsql::params![entry_id, tag_id, entity_id, updated_at, extra],
    )
    .await
    .map_err(AppError::LibSQL)?;
    Ok(())
}

async fn apply_task_tags_upsert(
    db: &Database,
    entity_id: &str,
    updated_at: i64,
    data: &serde_json::Value,
) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let obj = data.as_object().ok_or_else(|| AppError::Sync("task_tags: object expected".into()))?;
    let task_id = obj.get("task_id").and_then(|v| v.as_str()).unwrap_or_else(|| entity_id.split('|').next().unwrap_or(entity_id));
    let tag_id = obj.get("tag_id").and_then(|v| v.as_str()).unwrap_or_else(|| entity_id.split('|').nth(1).unwrap_or(""));
    let extra = obj.get("_extra").and_then(|v| v.as_str()).unwrap_or("{}");
    conn.execute(
        "INSERT OR REPLACE INTO task_tags (task_id, tag_id, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
        libsql::params![task_id, tag_id, entity_id, updated_at, extra],
    )
    .await
    .map_err(AppError::LibSQL)?;
    Ok(())
}

async fn apply_goal_tags_upsert(
    db: &Database,
    entity_id: &str,
    updated_at: i64,
    data: &serde_json::Value,
) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let obj = data.as_object().ok_or_else(|| AppError::Sync("goal_tags: object expected".into()))?;
    let goal_id = obj.get("goal_id").and_then(|v| v.as_str()).unwrap_or_else(|| entity_id.split('|').next().unwrap_or(entity_id));
    let tag_id = obj.get("tag_id").and_then(|v| v.as_str()).unwrap_or_else(|| entity_id.split('|').nth(1).unwrap_or(""));
    let extra = obj.get("_extra").and_then(|v| v.as_str()).unwrap_or("{}");
    conn.execute(
        "INSERT OR REPLACE INTO goal_tags (goal_id, tag_id, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
        libsql::params![goal_id, tag_id, entity_id, updated_at, extra],
    )
    .await
    .map_err(AppError::LibSQL)?;
    Ok(())
}

async fn apply_goal_instance_tags_upsert(
    db: &Database,
    entity_id: &str,
    updated_at: i64,
    data: &serde_json::Value,
) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let obj = data.as_object().ok_or_else(|| AppError::Sync("goal_instance_tags: object expected".into()))?;
    let goal_instance_id = obj.get("goal_instance_id").and_then(|v| v.as_str()).unwrap_or_else(|| entity_id.split('|').next().unwrap_or(entity_id));
    let tag_id = obj.get("tag_id").and_then(|v| v.as_str()).unwrap_or_else(|| entity_id.split('|').nth(1).unwrap_or(""));
    let extra = obj.get("_extra").and_then(|v| v.as_str()).unwrap_or("{}");
    conn.execute(
        "INSERT OR REPLACE INTO goal_instance_tags (goal_instance_id, tag_id, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
        libsql::params![goal_instance_id, tag_id, entity_id, updated_at, extra],
    )
    .await
    .map_err(AppError::LibSQL)?;
    Ok(())
}

async fn apply_bookmark_tags_upsert(
    db: &Database,
    entity_id: &str,
    updated_at: i64,
    data: &serde_json::Value,
) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let obj = data.as_object().ok_or_else(|| AppError::Sync("bookmark_tags: object expected".into()))?;
    let bookmark_id = obj.get("bookmark_id").and_then(|v| v.as_str()).unwrap_or_else(|| entity_id.split('|').next().unwrap_or(entity_id));
    let tag_id = obj.get("tag_id").and_then(|v| v.as_str()).unwrap_or_else(|| entity_id.split('|').nth(1).unwrap_or(""));
    let extra = obj.get("_extra").and_then(|v| v.as_str()).unwrap_or("{}");
    conn.execute(
        "INSERT OR REPLACE INTO bookmark_tags (bookmark_id, tag_id, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
        libsql::params![bookmark_id, tag_id, entity_id, updated_at, extra],
    )
    .await
    .map_err(AppError::LibSQL)?;
    Ok(())
}

async fn apply_goal_instances_upsert(
    db: &Database,
    entity_id: &str,
    updated_at: i64,
    data: &serde_json::Value,
) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let obj = data.as_object().ok_or_else(|| AppError::Sync("goal_instances: object expected".into()))?;
    let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or(entity_id);
    let goal_id = obj.get("goal_id").and_then(|v| v.as_str()).unwrap_or("");
    let period_start = obj.get("period_start").and_then(|v| v.as_str()).unwrap_or("");
    let period_end = obj.get("period_end").and_then(|v| v.as_str());
    let status = obj.get("status").and_then(|v| v.as_str()).unwrap_or("");
    let created_at = obj.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    let updated_at_s = obj.get("updated_at").and_then(|v| v.as_str());
    let deleted_at = obj.get("deleted_at").and_then(|v| v.as_str());
    let extra = obj.get("_extra").and_then(|v| v.as_str()).unwrap_or("{}");
    conn.execute(
        "INSERT OR REPLACE INTO goal_instances (id, goal_id, period_start, period_end, status, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, ?11)",
        libsql::params![id, goal_id, period_start, period_end, status, created_at, updated_at_s, deleted_at, entity_id, updated_at, extra],
    )
    .await
    .map_err(AppError::LibSQL)?;
    Ok(())
}

async fn apply_subtasks_upsert(
    db: &Database,
    entity_id: &str,
    updated_at: i64,
    data: &serde_json::Value,
) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let obj = data.as_object().ok_or_else(|| AppError::Sync("subtasks: object expected".into()))?;
    let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or(entity_id);
    let title = obj.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let is_completed = obj.get("is_completed").and_then(|v| v.as_bool()).unwrap_or(false);
    let task_id = obj.get("task_id").and_then(|v| v.as_str()).unwrap_or("");
    let order_index = obj.get("order_index").and_then(|v| v.as_i64()).unwrap_or(0);
    let created_at = obj.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    let updated_at_s = obj.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
    let deleted_at = obj.get("deleted_at").and_then(|v| v.as_str());
    let extra = obj.get("_extra").and_then(|v| v.as_str()).unwrap_or("{}");
    conn.execute(
        "INSERT OR REPLACE INTO subtasks (id, title, is_completed, task_id, order_index, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, ?11)",
        libsql::params![id, title, if is_completed { 1i64 } else { 0 }, task_id, order_index, created_at, updated_at_s, deleted_at, entity_id, updated_at, extra],
    )
    .await
    .map_err(AppError::LibSQL)?;
    Ok(())
}

async fn apply_tasks_upsert(
    db: &Database,
    entity_id: &str,
    updated_at: i64,
    data: &serde_json::Value,
) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let obj = data.as_object().ok_or_else(|| AppError::Sync("tasks: object expected".into()))?;
    let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or(entity_id);
    let title = obj.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let description = obj.get("description").and_then(|v| v.as_str());
    let is_completed = obj.get("is_completed").and_then(|v| v.as_bool()).unwrap_or(false);
    let due_date = obj.get("due_date").and_then(|v| v.as_str());
    let goal_instance_id = obj.get("goal_instance_id").and_then(|v| v.as_str());
    let goal_id = obj.get("goal_id").and_then(|v| v.as_str());
    let created_at = obj.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    let updated_at_s = obj.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
    let deleted_at = obj.get("deleted_at").and_then(|v| v.as_str());
    let extra = obj.get("_extra").and_then(|v| v.as_str()).unwrap_or("{}");

    conn.execute(
        "INSERT OR REPLACE INTO tasks (id, title, description, is_completed, due_date, goal_instance_id, goal_id, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 0, ?13)",
        libsql::params![
            id, title, description, if is_completed { 1i64 } else { 0 }, due_date,
            goal_instance_id, goal_id, created_at, updated_at_s, deleted_at,
            entity_id, updated_at, extra,
        ],
    )
    .await
    .map_err(AppError::LibSQL)?;
    Ok(())
}

async fn apply_goals_upsert(
    db: &Database,
    entity_id: &str,
    updated_at: i64,
    data: &serde_json::Value,
) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let obj = data.as_object().ok_or_else(|| AppError::Sync("goals: object expected".into()))?;
    let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or(entity_id);
    let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let description = obj.get("description").and_then(|v| v.as_str());
    let is_non_recurring = obj.get("is_non_recurring").and_then(|v| v.as_bool()).unwrap_or(true);
    let recurrence_type = obj.get("recurrence_type").and_then(|v| v.as_str());
    let recurrence_interval = obj.get("recurrence_interval").and_then(|v| v.as_i64());
    let recurrence_anchor = obj.get("recurrence_anchor").and_then(|v| v.as_str());
    let recurrence_meta = obj.get("recurrence_meta").and_then(|v| v.as_str());
    let timezone = obj.get("timezone").and_then(|v| v.as_str()).unwrap_or("UTC");
    let created_at = obj.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    let updated_at_s = obj.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
    let deleted_at = obj.get("deleted_at").and_then(|v| v.as_str());
    let extra = obj.get("_extra").and_then(|v| v.as_str()).unwrap_or("{}");

    conn.execute(
        "INSERT OR REPLACE INTO goals (id, name, description, is_non_recurring, recurrence_type, recurrence_interval, recurrence_anchor, recurrence_meta, timezone, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 0, ?15)",
        libsql::params![
            id, name, description, if is_non_recurring { 1i64 } else { 0 },
            recurrence_type, recurrence_interval, recurrence_anchor, recurrence_meta,
            timezone, created_at, updated_at_s, deleted_at, entity_id, updated_at, extra,
        ],
    )
    .await
    .map_err(AppError::LibSQL)?;
    Ok(())
}

async fn apply_canvases_upsert(
    db: &Database,
    entity_id: &str,
    updated_at: i64,
    data: &serde_json::Value,
) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let obj = data.as_object().ok_or_else(|| AppError::Sync("canvases: object expected".into()))?;
    let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or(entity_id);
    let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let canvas_data = obj.get("canvas_data").and_then(|v| v.as_str()).unwrap_or("{}");
    let created_at = obj.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    let updated_at_s = obj.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
    let deleted_at = obj.get("deleted_at").and_then(|v| v.as_str());
    let extra = obj.get("_extra").and_then(|v| v.as_str()).unwrap_or("{}");

    conn.execute(
        "INSERT OR REPLACE INTO canvases (id, name, canvas_data, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9)",
        libsql::params![id, name, canvas_data, created_at, updated_at_s, deleted_at, entity_id, updated_at, extra],
    )
    .await
    .map_err(AppError::LibSQL)?;
    Ok(())
}

async fn apply_bookmarks_upsert(
    db: &Database,
    entity_id: &str,
    updated_at: i64,
    data: &serde_json::Value,
) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let obj = data.as_object().ok_or_else(|| AppError::Sync("bookmarks: object expected".into()))?;
    let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or(entity_id);
    let url = obj.get("url").and_then(|v| v.as_str()).unwrap_or("");
    let title = obj.get("title").and_then(|v| v.as_str());
    let description = obj.get("description").and_then(|v| v.as_str());
    let image_url = obj.get("image_url").and_then(|v| v.as_str());
    let favicon_url = obj.get("favicon_url").and_then(|v| v.as_str());
    let site_name = obj.get("site_name").and_then(|v| v.as_str());
    let author = obj.get("author").and_then(|v| v.as_str());
    let published_at = obj.get("published_at").and_then(|v| v.as_str());
    let content_type = obj.get("content_type").and_then(|v| v.as_str());
    let metadata_json = obj.get("metadata_json").and_then(|v| v.as_str());
    let is_archived = obj.get("is_archived").and_then(|v| v.as_bool()).unwrap_or(false);
    let is_deleted = obj.get("is_deleted").and_then(|v| v.as_bool()).unwrap_or(false);
    let created_at = obj.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    let updated_at_s = obj.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
    let deleted_at = obj.get("deleted_at").and_then(|v| v.as_str());
    let extra = obj.get("_extra").and_then(|v| v.as_str()).unwrap_or("{}");

    conn.execute(
        "INSERT OR REPLACE INTO bookmarks (id, url, title, description, image_url, favicon_url, site_name, author, published_at, content_type, metadata_json, is_archived, is_deleted, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, 0, ?19)",
        libsql::params![
            id, url, title, description, image_url, favicon_url, site_name, author,
            published_at, content_type, metadata_json, if is_archived { 1i64 } else { 0 },
            if is_deleted { 1i64 } else { 0 }, created_at, updated_at_s, deleted_at,
            entity_id, updated_at, extra,
        ],
    )
    .await
    .map_err(AppError::LibSQL)?;
    Ok(())
}

async fn apply_media_items_upsert(
    db: &Database,
    entity_id: &str,
    updated_at: i64,
    data: &serde_json::Value,
) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let obj = data.as_object().ok_or_else(|| AppError::Sync("media_items: object expected".into()))?;
    let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or(entity_id);
    let entity_type = obj.get("entity_type").and_then(|v| v.as_str()).unwrap_or("entry");
    let entity_id_val = obj.get("entity_id").and_then(|v| v.as_str()).unwrap_or("");
    let media_type = obj.get("media_type").and_then(|v| v.as_str()).unwrap_or("audio");
    let file_path = obj.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
    let created_at = obj.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    let updated_at_s = obj.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
    let extra = obj.get("_extra").and_then(|v| v.as_str()).unwrap_or("{}");

    let mut meta: serde_json::Value =
        serde_json::from_str(obj.get("metadata").and_then(|v| v.as_str()).unwrap_or("{}"))
            .unwrap_or(serde_json::json!({}));
    if let Some(ch) = obj.get("content_hash").and_then(|v| v.as_str()) {
        meta["content_hash"] = serde_json::Value::String(ch.to_string());
    }
    let metadata = serde_json::to_string(&meta).map_err(AppError::Serialization)?;

    conn.execute(
        "INSERT OR REPLACE INTO media_items (id, entity_type, entity_id, media_type, file_path, metadata, created_at, updated_at, _sync_id, _updated_at, _deleted, _extra)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, ?11)",
        libsql::params![
            id, entity_type, entity_id_val, media_type, file_path, metadata, created_at, updated_at_s,
            entity_id, updated_at, extra,
        ],
    )
    .await
    .map_err(AppError::LibSQL)?;

    Ok(())
}

async fn apply_audio_transcriptions_upsert(
    db: &Database,
    entity_id: &str,
    updated_at: i64,
    data: &serde_json::Value,
) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let obj = data.as_object().ok_or_else(|| AppError::Sync("audio_transcriptions: object expected".into()))?;
    let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or(entity_id);
    let media_id = obj.get("media_id").and_then(|v| v.as_str()).unwrap_or("");
    let transcription_text = obj.get("transcription_text").and_then(|v| v.as_str()).unwrap_or("");
    let provider = obj.get("provider").and_then(|v| v.as_str()).unwrap_or("");
    let provider_config = obj.get("provider_config").and_then(|v| v.as_str());
    let confidence_score = obj.get("confidence_score").and_then(|v| v.as_f64());
    let status = obj.get("status").and_then(|v| v.as_str()).unwrap_or("pending");
    let error_message = obj.get("error_message").and_then(|v| v.as_str());
    let is_active = obj.get("is_active").and_then(|v| v.as_bool()).unwrap_or(false);
    let created_at = obj.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    let extra = obj.get("_extra").and_then(|v| v.as_str()).unwrap_or("{}");

    conn.execute(
        "INSERT OR REPLACE INTO audio_transcriptions (id, media_id, transcription_text, provider, provider_config, confidence_score, status, error_message, is_active, created_at, _sync_id, _updated_at, _deleted, _extra)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 0, ?13)",
        libsql::params![
            id, media_id, transcription_text, provider, provider_config, confidence_score,
            status, error_message, if is_active { 1i64 } else { 0 }, created_at,
            entity_id, updated_at, extra,
        ],
    )
    .await
    .map_err(AppError::LibSQL)?;
    Ok(())
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
