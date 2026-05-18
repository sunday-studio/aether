use crate::db::models::Canvas;
use crate::error::{AppError, Result};
use crate::utils::generate_id;
use chrono::Utc;
use libsql::Database;
use std::sync::Arc;

pub struct CanvasRepository {
    database: Arc<Database>,
}

impl CanvasRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Get all canvases (non-deleted)
    /// If limit and cursor are both None, returns all canvases (bypass pagination)
    /// Otherwise returns paginated results with cursor-based pagination
    pub async fn find_all(
        &self,
        limit: Option<u32>,
        cursor: Option<String>,
    ) -> Result<(Vec<Canvas>, Option<String>, bool)> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        // Bypass mode: return all results
        if limit.is_none() && cursor.is_none() {
            let mut rows = conn
                .query(
                    "SELECT id, name, canvas_data, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                     FROM canvases 
                     WHERE (deleted_at IS NULL) AND (_deleted = 0 OR _deleted IS NULL) 
                     ORDER BY id ASC",
                    libsql::params![],
                )
                .await
                .map_err(|e| AppError::LibSQL(e))?;

            let mut canvases = Vec::new();
            while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
                canvases.push(self.row_to_canvas(row)?);
            }

            return Ok((canvases, None, false));
        }

        // Pagination mode
        let limit_val = limit.unwrap_or(50).min(1000);
        let fetch_limit = limit_val + 1;

        let mut rows = if let Some(cursor_val) = cursor {
            use crate::commands::common::cursor;
            let last_id = cursor::decode(&cursor_val)?;

            conn.query(
                "SELECT id, name, canvas_data, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM canvases 
                 WHERE (deleted_at IS NULL) AND (_deleted = 0 OR _deleted IS NULL) AND id > ?1
                 ORDER BY id ASC
                 LIMIT ?2",
                libsql::params![last_id, fetch_limit as i64],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?
        } else {
            conn.query(
                "SELECT id, name, canvas_data, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM canvases 
                 WHERE (deleted_at IS NULL) AND (_deleted = 0 OR _deleted IS NULL) 
                 ORDER BY id ASC
                 LIMIT ?1",
                libsql::params![fetch_limit as i64],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?
        };

        let mut canvases = Vec::new();
        let mut has_more = false;

        let mut count = 0;
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            if count < limit_val {
                canvases.push(self.row_to_canvas(row)?);
                count += 1;
            } else {
                has_more = true;
                break;
            }
        }

        let next_cursor = if has_more && !canvases.is_empty() {
            use crate::commands::common::cursor;
            Some(cursor::encode(&canvases.last().unwrap().id))
        } else {
            None
        };

        Ok((canvases, next_cursor, has_more))
    }

    /// Get canvas by ID
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Canvas>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        let mut rows = conn
            .query(
                "SELECT id, name, canvas_data, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM canvases 
                 WHERE id = ?1 AND deleted_at IS NULL AND (_deleted = 0 OR _deleted IS NULL)",
                libsql::params![id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        if let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            Ok(Some(self.row_to_canvas(row)?))
        } else {
            Ok(None)
        }
    }

    /// Create a new canvas
    pub async fn create(&self, name: String, canvas_data: serde_json::Value) -> Result<Canvas> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        let id = generate_id("canvas");
        let now = Utc::now();
        let created_at_str = now.to_rfc3339();
        let updated_at_str = now.to_rfc3339();
        let canvas_data_str =
            serde_json::to_string(&canvas_data).map_err(|e| AppError::Serialization(e))?;
        let now_ms = now.timestamp_millis();

        conn.execute(
            "INSERT INTO canvases (id, name, canvas_data, created_at, updated_at, _sync_id, _updated_at, _deleted, _extra) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, '{}')",
            libsql::params![
                id.clone(),
                name.clone(),
                canvas_data_str,
                created_at_str,
                updated_at_str,
                id.clone(),
                now_ms,
            ],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(Canvas {
            id: id.clone(),
            name,
            canvas_data,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            _sync_id: Some(id),
            _updated_at: Some(now_ms),
            _deleted: false,
            _extra: None,
        })
    }

    /// Update a canvas
    pub async fn update(
        &self,
        id: &str,
        name: Option<String>,
        canvas_data: Option<serde_json::Value>,
    ) -> Result<Canvas> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        // Get current canvas
        let current = self.find_by_id(id).await?;
        let mut canvas =
            current.ok_or_else(|| AppError::NotFound(format!("Canvas {} not found", id)))?;

        // Update fields if provided
        if let Some(new_name) = name {
            canvas.name = new_name;
        }
        if let Some(new_canvas_data) = canvas_data {
            canvas.canvas_data = new_canvas_data;
        }
        canvas.updated_at = Utc::now();
        let now_ms = canvas.updated_at.timestamp_millis();

        let updated_at_str = canvas.updated_at.to_rfc3339();
        let canvas_data_str =
            serde_json::to_string(&canvas.canvas_data).map_err(|e| AppError::Serialization(e))?;

        conn.execute(
            "UPDATE canvases 
             SET name = ?1, canvas_data = ?2, updated_at = ?3, _updated_at = ?4 
             WHERE id = ?5",
            libsql::params![
                canvas.name.clone(),
                canvas_data_str,
                updated_at_str,
                now_ms,
                id
            ],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        canvas._updated_at = Some(now_ms);
        Ok(canvas)
    }

    /// Delete a canvas (soft delete)
    pub async fn delete(&self, id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        // Check if canvas exists
        let canvas = self.find_by_id(id).await?;
        if canvas.is_none() {
            return Err(AppError::NotFound(format!("Canvas {} not found", id)));
        }

        let now = Utc::now();
        let now_ms = now.timestamp_millis();
        let deleted_at_str = now.to_rfc3339();
        let updated_at_str = now.to_rfc3339();

        conn.execute(
            "UPDATE canvases SET deleted_at = ?1, updated_at = ?2, _deleted = 1, _updated_at = ?3 WHERE id = ?4",
            libsql::params![deleted_at_str, updated_at_str, now_ms, id],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Helper to convert database row to Canvas
    fn row_to_canvas(&self, row: libsql::Row) -> Result<Canvas> {
        let id: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
        let name: String = row.get(1).map_err(|e| AppError::LibSQL(e))?;
        let canvas_data_str: String = row.get(2).map_err(|e| AppError::LibSQL(e))?;
        let created_at_str: String = row.get(3).map_err(|e| AppError::LibSQL(e))?;
        let updated_at_str: String = row.get(4).map_err(|e| AppError::LibSQL(e))?;
        let deleted_at_str: Option<String> = row.get(5).map_err(|e| AppError::LibSQL(e))?;
        let _sync_id: Option<String> = row.get(6).ok();
        let _updated_at: Option<i64> = row.get(7).ok();
        let _deleted: i64 = row.get(8).unwrap_or(0);
        let _extra: Option<serde_json::Value> = row
            .get::<Option<String>>(9)
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str(&s).ok());

        let canvas_data =
            serde_json::from_str(&canvas_data_str).map_err(|e| AppError::Serialization(e))?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid updated_at: {}", e)))?
            .with_timezone(&Utc);
        let deleted_at = deleted_at_str
            .map(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .flatten()
            .map(|dt| dt.with_timezone(&Utc));

        Ok(Canvas {
            id,
            name,
            canvas_data,
            created_at,
            updated_at,
            deleted_at,
            _sync_id,
            _updated_at,
            _deleted: _deleted != 0,
            _extra,
        })
    }
}
