//! Helpers to enqueue changes to _sync_outbox when sync triggers are suppressed.

use crate::error::{AppError, Result};
use libsql::Database;
use std::sync::Arc;

/// Enqueue one row into _sync_outbox (call when _suppress_triggers is '1' to avoid double-queueing).
pub async fn enqueue(db: Arc<Database>, entity: &str, entity_id: &str, op: &str) -> Result<()> {
    let queued_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let conn = db.connect().map_err(AppError::LibSQL)?;
    conn.execute(
        "INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at) VALUES (?1, ?2, ?3, ?4)",
        libsql::params![entity, entity_id, op, queued_at],
    )
    .await
    .map_err(AppError::LibSQL)?;
    Ok(())
}
