use crate::db::models::Tag;
use crate::error::{AppError, Result};
use crate::utils::generate_id;
use chrono::Utc;
use libsql::Database;
use std::sync::Arc;

pub struct TagRepository {
    database: Arc<Database>,
}

impl TagRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Get all tags
    pub async fn find_all(&self) -> Result<Vec<Tag>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query("SELECT id, name, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra FROM tags WHERE deleted_at IS NULL ORDER BY name ASC LIMIT 1000", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut tags = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            let id: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
            let name: String = row.get(1).map_err(|e| AppError::LibSQL(e))?;
            let created_at_str: String = row.get(2).map_err(|e| AppError::LibSQL(e))?;
            let updated_at_str: String = row.get(3).map_err(|e| AppError::LibSQL(e))?;
            let deleted_at_str: Option<String> = row.get(4).map_err(|e| AppError::LibSQL(e))?;
            let _sync_id: Option<String> = row.get(5).ok();
            let _updated_at: Option<i64> = row.get(6).ok();
            let _deleted: i64 = row.get(7).unwrap_or(0);
            let _extra: Option<serde_json::Value> = row.get::<Option<String>>(8).ok().flatten().and_then(|s| serde_json::from_str(&s).ok());

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

            tags.push(Tag {
                id,
                name,
                created_at,
                updated_at,
                deleted_at,
                _sync_id,
                _updated_at,
                _deleted: _deleted != 0,
                _extra,
            });
        }

        Ok(tags)
    }

    /// Create a new tag
    pub async fn create(&self, name: String) -> Result<Tag> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let id = generate_id("tag");
        let now = Utc::now();
        let created_at = now.to_rfc3339();
        let updated_at = now.to_rfc3339();

        let now_ms = now.timestamp_millis();
        conn.execute(
            "INSERT INTO tags (id, name, created_at, updated_at, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, '{}')",
            libsql::params![id.clone(), name.clone(), created_at, updated_at, id.clone(), now_ms],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(Tag {
            id: id.clone(),
            name,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            _sync_id: Some(id),
            _updated_at: Some(now_ms),
            _deleted: false,
            _extra: None,
        })
    }

    /// Bulk create tags
    pub async fn bulk_create(&self, names: Vec<String>) -> Result<Vec<Tag>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Use a transaction for bulk operations
        conn.execute("BEGIN TRANSACTION", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut tags = Vec::new();
        let now = Utc::now();
        let created_at = now.to_rfc3339();
        let updated_at = now.to_rfc3339();

        let now_ms = now.timestamp_millis();
        for name in names {
            let id = generate_id("tag");
            
            conn.execute(
                "INSERT INTO tags (id, name, created_at, updated_at, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, '{}')",
                libsql::params![id.clone(), name.clone(), created_at.clone(), updated_at.clone(), id.clone(), now_ms],
            )
            .await
            .map_err(|e| {
                let _ = conn.execute("ROLLBACK", libsql::params![]);
                AppError::LibSQL(e)
            })?;

            tags.push(Tag {
                id: id.clone(),
                name,
                created_at: now,
                updated_at: now,
                deleted_at: None,
                _sync_id: Some(id),
                _updated_at: Some(now_ms),
                _deleted: false,
                _extra: None,
            });
        }

        conn.execute("COMMIT", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        Ok(tags)
    }
}
