use crate::db::models::{Bookmark, Tag};
use crate::error::{AppError, Result};
use crate::utils::{generate_id, embeddings::generate_embedding};
use chrono::Utc;
use libsql::Database;
use std::sync::Arc;

pub struct BookmarkRepository {
    database: Arc<Database>,
}

impl BookmarkRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Get all bookmarks (non-deleted)
    /// If limit and cursor are both None, returns all bookmarks (bypass pagination)
    /// Otherwise returns paginated results with cursor-based pagination
    pub async fn find_all(
        &self,
        is_archived: Option<bool>,
        tag_ids: Option<Vec<String>>,
        content_type: Option<String>,
        limit: Option<u32>,
        cursor: Option<String>,
    ) -> Result<(Vec<Bookmark>, Option<String>, bool)> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut query = "SELECT id, url, title, description, image_url, favicon_url, site_name, author, published_at, content_type, metadata_json, is_archived, is_deleted, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra FROM bookmarks WHERE is_deleted = 0 AND (_deleted = 0 OR _deleted IS NULL)".to_string();

        // Build query with filters
        if let Some(archived) = is_archived {
            query.push_str(&format!(" AND is_archived = {}", if archived { 1 } else { 0 }));
        }

        if let Some(ref content_type_filter) = content_type {
            query.push_str(&format!(" AND content_type = '{}'", content_type_filter.replace("'", "''")));
        }

        if let Some(ref tag_ids_filter) = tag_ids {
            if !tag_ids_filter.is_empty() {
                let escaped_ids: Vec<String> = tag_ids_filter.iter()
                    .map(|id| format!("'{}'", id.replace("'", "''")))
                    .collect();
                query.push_str(&format!(
                    " AND id IN (SELECT bookmark_id FROM bookmark_tags WHERE tag_id IN ({}))",
                    escaped_ids.join(", ")
                ));
            }
        }

        // Bypass mode: return all results
        if limit.is_none() && cursor.is_none() {
            query.push_str(" ORDER BY id ASC");
            let mut rows = conn
                .query(&query, libsql::params![])
                .await
                .map_err(|e| AppError::LibSQL(e))?;

            let mut bookmarks = Vec::new();
            while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
                bookmarks.push(self.row_to_bookmark(row)?);
            }

            return Ok((bookmarks, None, false));
        }

        // Pagination mode
        let limit_val = limit.unwrap_or(50).min(1000);
        let fetch_limit = limit_val + 1;
        
        // Add cursor condition if provided
        if let Some(cursor_val) = cursor {
            use crate::commands::common::cursor;
            let last_id = cursor::decode(&cursor_val)?;
            query.push_str(&format!(" AND id > '{}'", last_id.replace("'", "''")));
        }

        query.push_str(" ORDER BY id ASC");
        query.push_str(&format!(" LIMIT {}", fetch_limit));

        let mut rows = conn
            .query(&query, libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut bookmarks = Vec::new();
        let mut has_more = false;
        
        let mut count = 0;
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            if count < limit_val {
                bookmarks.push(self.row_to_bookmark(row)?);
                count += 1;
            } else {
                has_more = true;
                break;
            }
        }

        let next_cursor = if has_more && !bookmarks.is_empty() {
            use crate::commands::common::cursor;
            Some(cursor::encode(&bookmarks.last().unwrap().id))
        } else {
            None
        };

        Ok((bookmarks, next_cursor, has_more))
    }

    /// Get bookmark by ID
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Bookmark>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT id, url, title, description, image_url, favicon_url, site_name, author, published_at, content_type, metadata_json, is_archived, is_deleted, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM bookmarks 
                 WHERE id = ?1 AND is_deleted = 0 AND (_deleted = 0 OR _deleted IS NULL)",
                libsql::params![id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        if let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            Ok(Some(self.row_to_bookmark(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get bookmark by URL (for duplicate detection)
    pub async fn find_by_url(&self, url: &str) -> Result<Option<Bookmark>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT id, url, title, description, image_url, favicon_url, site_name, author, published_at, content_type, metadata_json, is_archived, is_deleted, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM bookmarks 
                 WHERE url = ?1 AND is_deleted = 0 AND (_deleted = 0 OR _deleted IS NULL)",
                libsql::params![url],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        if let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            Ok(Some(self.row_to_bookmark(row)?))
        } else {
            Ok(None)
        }
    }

    /// Create a new bookmark
    pub async fn create(
        &self,
        url: String,
        title: Option<String>,
        description: Option<String>,
        image_url: Option<String>,
        favicon_url: Option<String>,
        site_name: Option<String>,
        author: Option<String>,
        published_at: Option<chrono::DateTime<Utc>>,
        content_type: Option<String>,
        metadata_json: Option<serde_json::Value>,
    ) -> Result<Bookmark> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Check for duplicate URL
        if let Some(_existing) = self.find_by_url(&url).await? {
            return Err(AppError::BadRequest(format!("Bookmark with URL {} already exists", url)));
        }

        let id = generate_id("bookmark");
        let now = Utc::now();
        let created_at_str = now.to_rfc3339();
        let updated_at_str = now.to_rfc3339();
        let published_at_str = published_at.map(|d| d.to_rfc3339());
        let metadata_json_str = metadata_json.as_ref().and_then(|v| serde_json::to_string(v).ok());
        let now_ms = now.timestamp_millis();

        conn.execute(
            "INSERT INTO bookmarks (id, url, title, description, image_url, favicon_url, site_name, author, published_at, content_type, metadata_json, is_archived, is_deleted, created_at, updated_at, _sync_id, _updated_at, _deleted, _extra) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, 0, '{}')",
            libsql::params![
                id.clone(),
                url.clone(),
                title.as_ref().map(|s| s.as_str()),
                description.as_ref().map(|s| s.as_str()),
                image_url.as_ref().map(|s| s.as_str()),
                favicon_url.as_ref().map(|s| s.as_str()),
                site_name.as_ref().map(|s| s.as_str()),
                author.as_ref().map(|s| s.as_str()),
                published_at_str.as_ref().map(|s| s.as_str()),
                content_type.as_ref().map(|s| s.as_str()),
                metadata_json_str.as_ref().map(|s| s.as_str()),
                0, // is_archived
                0, // is_deleted
                created_at_str,
                updated_at_str,
                id.clone(),
                now_ms,
            ],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        let bookmark = Bookmark {
            id: id.clone(),
            url,
            title,
            description,
            image_url,
            favicon_url,
            site_name,
            author,
            published_at,
            content_type,
            metadata_json,
            is_archived: false,
            is_deleted: false,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            _sync_id: Some(id.clone()),
            _updated_at: Some(now_ms),
            _deleted: false,
            _extra: None,
        };

        // Generate embedding for semantic search
        if let Err(e) = self.generate_embedding(&id).await {
            tracing::warn!("Failed to generate embedding for bookmark {}: {}", id, e);
        }

        Ok(bookmark)
    }

    /// Update a bookmark
    pub async fn update(
        &self,
        id: &str,
        title: Option<String>,
        description: Option<String>,
        image_url: Option<String>,
        favicon_url: Option<String>,
        site_name: Option<String>,
        author: Option<String>,
        published_at: Option<chrono::DateTime<Utc>>,
        content_type: Option<String>,
        metadata_json: Option<serde_json::Value>,
        is_archived: Option<bool>,
    ) -> Result<Bookmark> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Get current bookmark
        let current = self.find_by_id(id).await?;
        let mut bookmark = current.ok_or_else(|| AppError::NotFound(format!("Bookmark {} not found", id)))?;

        // Update fields
        if let Some(t) = title {
            bookmark.title = Some(t);
        }
        if let Some(d) = description {
            bookmark.description = Some(d);
        }
        if let Some(img) = image_url {
            bookmark.image_url = Some(img);
        }
        if let Some(fav) = favicon_url {
            bookmark.favicon_url = Some(fav);
        }
        if let Some(sn) = site_name {
            bookmark.site_name = Some(sn);
        }
        if let Some(a) = author {
            bookmark.author = Some(a);
        }
        if let Some(pa) = published_at {
            bookmark.published_at = Some(pa);
        }
        if let Some(ct) = content_type {
            bookmark.content_type = Some(ct);
        }
        if let Some(mj) = metadata_json {
            bookmark.metadata_json = Some(mj);
        }
        if let Some(archived) = is_archived {
            bookmark.is_archived = archived;
        }
        bookmark.updated_at = Utc::now();
        let now_ms = bookmark.updated_at.timestamp_millis();

        let updated_at_str = bookmark.updated_at.to_rfc3339();
        let published_at_str = bookmark.published_at.map(|d| d.to_rfc3339());
        let metadata_json_str = bookmark.metadata_json.as_ref().and_then(|v| serde_json::to_string(v).ok());

        conn.execute(
            "UPDATE bookmarks 
             SET title = ?1, description = ?2, image_url = ?3, favicon_url = ?4, site_name = ?5, author = ?6, published_at = ?7, content_type = ?8, metadata_json = ?9, is_archived = ?10, updated_at = ?11, _updated_at = ?12 
             WHERE id = ?13",
            libsql::params![
                bookmark.title.as_ref().map(|s| s.as_str()),
                bookmark.description.as_ref().map(|s| s.as_str()),
                bookmark.image_url.as_ref().map(|s| s.as_str()),
                bookmark.favicon_url.as_ref().map(|s| s.as_str()),
                bookmark.site_name.as_ref().map(|s| s.as_str()),
                bookmark.author.as_ref().map(|s| s.as_str()),
                published_at_str.as_ref().map(|s| s.as_str()),
                bookmark.content_type.as_ref().map(|s| s.as_str()),
                metadata_json_str.as_ref().map(|s| s.as_str()),
                if bookmark.is_archived { 1 } else { 0 },
                updated_at_str,
                now_ms,
                id
            ],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        bookmark._updated_at = Some(now_ms);

        // Regenerate embedding if content changed
        if let Err(e) = self.generate_embedding(id).await {
            tracing::warn!("Failed to regenerate embedding for bookmark {}: {}", id, e);
        }

        Ok(bookmark)
    }

    /// Delete a bookmark (soft delete)
    pub async fn delete(&self, id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Check if bookmark exists
        let bookmark = self.find_by_id(id).await?;
        if bookmark.is_none() {
            return Err(AppError::NotFound(format!("Bookmark {} not found", id)));
        }

        let now = Utc::now();
        let now_ms = now.timestamp_millis();
        let updated_at_str = now.to_rfc3339();
        let deleted_at_str = now.to_rfc3339();

        conn.execute(
            "UPDATE bookmarks SET is_deleted = 1, updated_at = ?1, deleted_at = ?2, _deleted = 1, _updated_at = ?3 WHERE id = ?4",
            libsql::params![updated_at_str, deleted_at_str, now_ms, id],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Add tags to a bookmark
    pub async fn add_tags(&self, bookmark_id: &str, tag_ids: Vec<String>) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Verify bookmark exists
        let bookmark = self.find_by_id(bookmark_id).await?;
        if bookmark.is_none() {
            return Err(AppError::NotFound(format!("Bookmark {} not found", bookmark_id)));
        }

        if tag_ids.is_empty() {
            return Ok(());
        }

        // Verify tags exist
        for tag_id in &tag_ids {
            let mut rows = conn
                .query("SELECT id FROM tags WHERE id = ?1", libsql::params![tag_id.as_str()])
                .await
                .map_err(|e| AppError::LibSQL(e))?;
            
            if rows.next().await.map_err(|e| AppError::LibSQL(e))?.is_none() {
                return Err(AppError::NotFound(format!("Tag {} not found", tag_id)));
            }
        }

        // Insert tag associations (skip if already exists)
        let now_ms = Utc::now().timestamp_millis();
        conn.execute("BEGIN TRANSACTION", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        for tag_id in &tag_ids {
            let sync_id = format!("{}|{}", bookmark_id, tag_id);
            conn.execute(
                "INSERT OR IGNORE INTO bookmark_tags (bookmark_id, tag_id, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, 0, '{}')",
                libsql::params![bookmark_id, tag_id.as_str(), sync_id, now_ms],
            )
            .await
            .map_err(|e| {
                let _ = conn.execute("ROLLBACK", libsql::params![]);
                AppError::LibSQL(e)
            })?;
        }

        conn.execute("COMMIT", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Remove tags from a bookmark
    pub async fn remove_tags(&self, bookmark_id: &str, tag_ids: Vec<String>) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Verify bookmark exists
        let bookmark = self.find_by_id(bookmark_id).await?;
        if bookmark.is_none() {
            return Err(AppError::NotFound(format!("Bookmark {} not found", bookmark_id)));
        }

        if tag_ids.is_empty() {
            return Ok(());
        }

        // Remove tag associations
        conn.execute("BEGIN TRANSACTION", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        for tag_id in &tag_ids {
            conn.execute(
                "DELETE FROM bookmark_tags WHERE bookmark_id = ?1 AND tag_id = ?2",
                libsql::params![bookmark_id, tag_id.as_str()],
            )
            .await
            .map_err(|e| {
                let _ = conn.execute("ROLLBACK", libsql::params![]);
                AppError::LibSQL(e)
            })?;
        }

        conn.execute("COMMIT", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Get tags for a bookmark
    pub async fn get_tags(&self, bookmark_id: &str) -> Result<Vec<Tag>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT t.id, t.name, t.created_at, t.updated_at, t.deleted_at, t._sync_id, t._updated_at, t._deleted, t._extra
                 FROM tags t
                 INNER JOIN bookmark_tags bt ON t.id = bt.tag_id
                 WHERE bt.bookmark_id = ?1 AND t.deleted_at IS NULL
                 ORDER BY t.name ASC",
                libsql::params![bookmark_id],
            )
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

    /// Generate embedding for a bookmark
    pub async fn generate_embedding(&self, bookmark_id: &str) -> Result<()> {
        let bookmark = self.find_by_id(bookmark_id).await?
            .ok_or_else(|| AppError::NotFound(format!("Bookmark {} not found", bookmark_id)))?;

        // Build text for embedding (title + description + site_name)
        let mut text_parts = Vec::new();
        if let Some(ref title) = bookmark.title {
            text_parts.push(title.clone());
        }
        if let Some(ref description) = bookmark.description {
            text_parts.push(description.clone());
        }
        if let Some(ref site_name) = bookmark.site_name {
            text_parts.push(site_name.clone());
        }

        if text_parts.is_empty() {
            return Ok(()); // Nothing to embed
        }

        let text = text_parts.join(" ");
        let embedding = generate_embedding(&text).await?;
        let embedding_json = serde_json::to_string(&embedding)
            .map_err(|e| AppError::Internal(format!("Failed to serialize embedding: {}", e)))?;

        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        conn.execute(
            "UPDATE bookmarks SET embedding = vector32(?1) WHERE id = ?2",
            libsql::params![embedding_json, bookmark_id],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Helper to convert database row to Bookmark
    fn row_to_bookmark(&self, row: libsql::Row) -> Result<Bookmark> {
        let id: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
        let url: String = row.get(1).map_err(|e| AppError::LibSQL(e))?;
        let title: Option<String> = row.get(2).map_err(|e| AppError::LibSQL(e))?;
        let description: Option<String> = row.get(3).map_err(|e| AppError::LibSQL(e))?;
        let image_url: Option<String> = row.get(4).map_err(|e| AppError::LibSQL(e))?;
        let favicon_url: Option<String> = row.get(5).map_err(|e| AppError::LibSQL(e))?;
        let site_name: Option<String> = row.get(6).map_err(|e| AppError::LibSQL(e))?;
        let author: Option<String> = row.get(7).map_err(|e| AppError::LibSQL(e))?;
        let published_at_str: Option<String> = row.get(8).map_err(|e| AppError::LibSQL(e))?;
        let content_type: Option<String> = row.get(9).map_err(|e| AppError::LibSQL(e))?;
        let metadata_json_str: Option<String> = row.get(10).map_err(|e| AppError::LibSQL(e))?;
        let is_archived: i64 = row.get(11).map_err(|e| AppError::LibSQL(e))?;
        let is_deleted: i64 = row.get(12).map_err(|e| AppError::LibSQL(e))?;
        let created_at_str: String = row.get(13).map_err(|e| AppError::LibSQL(e))?;
        let updated_at_str: String = row.get(14).map_err(|e| AppError::LibSQL(e))?;
        let deleted_at_str: Option<String> = row.get(15).map_err(|e| AppError::LibSQL(e))?;
        let _sync_id: Option<String> = row.get(16).ok();
        let _updated_at: Option<i64> = row.get(17).ok();
        let _deleted: i64 = row.get(18).unwrap_or(0);
        let _extra: Option<serde_json::Value> = row.get::<Option<String>>(19).ok().flatten().and_then(|s| serde_json::from_str(&s).ok());

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
        let published_at = published_at_str
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        let metadata_json = metadata_json_str
            .and_then(|s| serde_json::from_str(&s).ok());

        Ok(Bookmark {
            id,
            url,
            title,
            description,
            image_url,
            favicon_url,
            site_name,
            author,
            published_at,
            content_type,
            metadata_json,
            is_archived: is_archived != 0,
            is_deleted: is_deleted != 0,
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
