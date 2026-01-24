use crate::db::models::MediaItem;
use crate::error::{AppError, Result};
use crate::utils::generate_id;
use chrono::Utc;
use libsql::Database;
use std::sync::Arc;

pub struct MediaRepository {
    database: Arc<Database>,
}

impl MediaRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Create a new media item
    pub async fn create(
        &self,
        entity_type: String,
        entity_id: String,
        media_type: String,
        file_path: String,
        metadata: serde_json::Value,
    ) -> Result<MediaItem> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let id = generate_id("media");
        let now = Utc::now();
        let created_at_str = now.to_rfc3339();
        let updated_at_str = now.to_rfc3339();
        let metadata_str = serde_json::to_string(&metadata)
            .map_err(|e| AppError::Serialization(e))?;

        let now_ms = now.timestamp_millis();
        conn.execute(
            "INSERT INTO media_items (id, entity_type, entity_id, media_type, file_path, metadata, created_at, updated_at, _sync_id, _updated_at, _deleted, _extra) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, '{}')",
            libsql::params![
                id.clone(),
                entity_type.clone(),
                entity_id.clone(),
                media_type.clone(),
                file_path.clone(),
                metadata_str,
                created_at_str,
                updated_at_str,
                id.clone(),
                now_ms,
            ],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(MediaItem {
            id: id.clone(),
            entity_type,
            entity_id,
            media_type,
            file_path,
            metadata,
            created_at: now,
            updated_at: now,
            _sync_id: Some(id),
            _updated_at: Some(now_ms),
            _deleted: false,
            _extra: None,
        })
    }

    /// Get media item by ID
    pub async fn find_by_id(&self, id: &str) -> Result<Option<MediaItem>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT id, entity_type, entity_id, media_type, file_path, metadata, created_at, updated_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM media_items 
                 WHERE id = ?1 AND (_deleted = 0 OR _deleted IS NULL)",
                libsql::params![id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        if let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            Ok(Some(self.row_to_media_item(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get all media items for an entity
    pub async fn find_by_entity(&self, entity_type: &str, entity_id: &str) -> Result<Vec<MediaItem>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT id, entity_type, entity_id, media_type, file_path, metadata, created_at, updated_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM media_items 
                 WHERE entity_type = ?1 AND entity_id = ?2 AND (_deleted = 0 OR _deleted IS NULL) 
                 ORDER BY created_at ASC",
                libsql::params![entity_type, entity_id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut items = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            items.push(self.row_to_media_item(row)?);
        }

        Ok(items)
    }

    /// Get all media items for an entry (backward compatibility)
    pub async fn find_by_entry_id(&self, entry_id: &str) -> Result<Vec<MediaItem>> {
        self.find_by_entity("entry", entry_id).await
    }

    /// Get file path for a media item
    pub async fn get_file_path(&self, id: &str) -> Result<Option<String>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT file_path FROM media_items WHERE id = ?1 AND (_deleted = 0 OR _deleted IS NULL)",
                libsql::params![id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        if let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            let file_path: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
            Ok(Some(file_path))
        } else {
            Ok(None)
        }
    }

    /// Delete a media item
    pub async fn delete(&self, id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Check if media item exists
        let item = self.find_by_id(id).await?;
        if item.is_none() {
            return Err(AppError::NotFound(format!("Media item {} not found", id)));
        }

        let now_ms = Utc::now().timestamp_millis();
        conn.execute(
            "UPDATE media_items SET _deleted = 1, _updated_at = ?1 WHERE id = ?2",
            libsql::params![now_ms, id],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Helper to convert database row to MediaItem
    fn row_to_media_item(&self, row: libsql::Row) -> Result<MediaItem> {
        let id: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
        let entity_type: String = row.get(1).map_err(|e| AppError::LibSQL(e))?;
        let entity_id: String = row.get(2).map_err(|e| AppError::LibSQL(e))?;
        let media_type: String = row.get(3).map_err(|e| AppError::LibSQL(e))?;
        let file_path: String = row.get(4).map_err(|e| AppError::LibSQL(e))?;
        let metadata_str: String = row.get(5).map_err(|e| AppError::LibSQL(e))?;
        let created_at_str: String = row.get(6).map_err(|e| AppError::LibSQL(e))?;
        let updated_at_str: String = row.get(7).map_err(|e| AppError::LibSQL(e))?;
        let _sync_id: Option<String> = row.get(8).ok();
        let _updated_at: Option<i64> = row.get(9).ok();
        let _deleted: i64 = row.get(10).unwrap_or(0);
        let _extra: Option<serde_json::Value> = row.get::<Option<String>>(11).ok().flatten().and_then(|s| serde_json::from_str(&s).ok());

        let metadata = serde_json::from_str(&metadata_str)
            .map_err(|e| AppError::Serialization(e))?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid updated_at: {}", e)))?
            .with_timezone(&Utc);

        Ok(MediaItem {
            id,
            entity_type,
            entity_id,
            media_type,
            file_path,
            metadata,
            created_at,
            updated_at,
            _sync_id,
            _updated_at,
            _deleted: _deleted != 0,
            _extra,
        })
    }
}
