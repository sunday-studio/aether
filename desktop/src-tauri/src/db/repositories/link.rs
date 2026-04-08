use crate::db::models::ResourceLink;
use crate::error::{AppError, Result};
use crate::utils::generate_id;
use chrono::Utc;
use libsql::Database;
use std::sync::Arc;

pub struct LinkRepository {
    database: Arc<Database>,
}

impl LinkRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Create a new link
    pub async fn create(
        &self,
        source_type: String,
        source_id: String,
        target_type: String,
        target_id: String,
        link_text: Option<String>,
    ) -> Result<ResourceLink> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        let now = Utc::now();
        let created_at_str = now.to_rfc3339();
        let now_ms = now.timestamp_millis();

        let mut existing_rows = conn
            .query(
                "SELECT id, created_at, COALESCE(_deleted, 0) FROM resource_links
                 WHERE source_type = ?1 AND source_id = ?2 AND target_type = ?3 AND target_id = ?4",
                libsql::params![
                    source_type.clone(),
                    source_id.clone(),
                    target_type.clone(),
                    target_id.clone()
                ],
            )
            .await
            .map_err(AppError::LibSQL)?;

        if let Some(row) = existing_rows.next().await.map_err(AppError::LibSQL)? {
            let existing_id: String = row.get(0).map_err(AppError::LibSQL)?;
            let existing_created_at: String = row.get(1).map_err(AppError::LibSQL)?;
            let deleted: i64 = row.get(2).map_err(AppError::LibSQL)?;

            if deleted == 0 {
                return Err(AppError::BadRequest(format!(
                    "Link already exists from {}:{} to {}:{}",
                    source_type, source_id, target_type, target_id
                )));
            }

            conn.execute(
                "UPDATE resource_links
                 SET link_text = ?1, _deleted = 0, _updated_at = ?2, _sync_id = COALESCE(_sync_id, id), _extra = COALESCE(_extra, '{}')
                 WHERE id = ?3",
                libsql::params![link_text.as_deref(), now_ms, existing_id.clone()],
            )
            .await
            .map_err(AppError::LibSQL)?;

            let created_at = chrono::DateTime::parse_from_rfc3339(&existing_created_at)
                .map_err(|e| AppError::Internal(format!("Invalid created_at: {}", e)))?
                .with_timezone(&Utc);

            return Ok(ResourceLink {
                id: existing_id,
                source_type,
                source_id,
                target_type,
                target_id,
                link_text,
                created_at,
            });
        }

        let id = generate_id("link");

        conn.execute(
            "INSERT INTO resource_links (id, source_type, source_id, target_type, target_id, link_text, created_at, _sync_id, _updated_at, _deleted, _extra)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, '{}')",
            libsql::params![
                id.clone(),
                source_type.clone(),
                source_id.clone(),
                target_type.clone(),
                target_id.clone(),
                link_text.as_ref().map(|s| s.as_str()),
                created_at_str,
                id.clone(),
                now_ms,
            ],
        )
        .await
        .map_err(AppError::LibSQL)?;

        Ok(ResourceLink {
            id,
            source_type,
            source_id,
            target_type,
            target_id,
            link_text,
            created_at: now,
        })
    }

    /// Get all links from a source resource
    pub async fn find_by_source(
        &self,
        source_type: &str,
        source_id: &str,
    ) -> Result<Vec<ResourceLink>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        let mut rows = conn
            .query(
                "SELECT id, source_type, source_id, target_type, target_id, link_text, created_at 
                 FROM resource_links 
                 WHERE source_type = ?1 AND source_id = ?2 AND (_deleted = 0 OR _deleted IS NULL)
                 ORDER BY created_at DESC",
                libsql::params![source_type, source_id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut links = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            links.push(self.row_to_link(row)?);
        }

        Ok(links)
    }

    /// Get all backlinks to a target resource
    pub async fn find_by_target(
        &self,
        target_type: &str,
        target_id: &str,
    ) -> Result<Vec<ResourceLink>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        let mut rows = conn
            .query(
                "SELECT id, source_type, source_id, target_type, target_id, link_text, created_at 
                 FROM resource_links 
                 WHERE target_type = ?1 AND target_id = ?2 AND (_deleted = 0 OR _deleted IS NULL)
                 ORDER BY created_at DESC",
                libsql::params![target_type, target_id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut links = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            links.push(self.row_to_link(row)?);
        }

        Ok(links)
    }

    /// Delete a specific link
    pub async fn delete(
        &self,
        source_type: &str,
        source_id: &str,
        target_type: &str,
        target_id: &str,
    ) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        let now_ms = Utc::now().timestamp_millis();

        conn.execute(
            "UPDATE resource_links
             SET _deleted = 1, _updated_at = ?1
             WHERE source_type = ?2 AND source_id = ?3 AND target_type = ?4 AND target_id = ?5
               AND (_deleted = 0 OR _deleted IS NULL)",
            libsql::params![now_ms, source_type, source_id, target_type, target_id],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Delete all links from a source resource (when resource is deleted)
    pub async fn delete_by_source(&self, source_type: &str, source_id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        let now_ms = Utc::now().timestamp_millis();

        conn.execute(
            "UPDATE resource_links
             SET _deleted = 1, _updated_at = ?1
             WHERE source_type = ?2 AND source_id = ?3
               AND (_deleted = 0 OR _deleted IS NULL)",
            libsql::params![now_ms, source_type, source_id],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Delete all links to a target resource (when resource is deleted)
    pub async fn delete_by_target(&self, target_type: &str, target_id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        let now_ms = Utc::now().timestamp_millis();

        conn.execute(
            "UPDATE resource_links
             SET _deleted = 1, _updated_at = ?1
             WHERE target_type = ?2 AND target_id = ?3
               AND (_deleted = 0 OR _deleted IS NULL)",
            libsql::params![now_ms, target_type, target_id],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Get all links for graph visualization
    /// Get all links for graph visualization
    /// If limit and cursor are both None, returns all links (bypass pagination)
    /// Otherwise returns paginated results with cursor-based pagination
    pub async fn find_all_for_graph(
        &self,
        limit: Option<u32>,
        cursor: Option<String>,
    ) -> Result<(Vec<ResourceLink>, Option<String>, bool)> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        // Bypass mode: return all results
        if limit.is_none() && cursor.is_none() {
            let mut rows = conn
                .query(
                    "SELECT id, source_type, source_id, target_type, target_id, link_text, created_at
                     FROM resource_links
                     WHERE (_deleted = 0 OR _deleted IS NULL)
                     ORDER BY id ASC",
                    libsql::params![],
                )
                .await
                .map_err(|e| AppError::LibSQL(e))?;

            let mut links = Vec::new();
            while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
                links.push(self.row_to_link(row)?);
            }

            return Ok((links, None, false));
        }

        // Pagination mode
        let limit_val = limit.unwrap_or(50).min(1000);
        let fetch_limit = limit_val + 1;

        let mut rows = if let Some(cursor_val) = cursor {
            use crate::commands::common::cursor;
            let last_id = cursor::decode(&cursor_val)?;

            conn.query(
                "SELECT id, source_type, source_id, target_type, target_id, link_text, created_at 
                 FROM resource_links 
                 WHERE id > ?1 AND (_deleted = 0 OR _deleted IS NULL)
                 ORDER BY id ASC
                 LIMIT ?2",
                libsql::params![last_id, fetch_limit as i64],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?
        } else {
            conn.query(
                "SELECT id, source_type, source_id, target_type, target_id, link_text, created_at 
                 FROM resource_links 
                 WHERE (_deleted = 0 OR _deleted IS NULL)
                 ORDER BY id ASC
                 LIMIT ?1",
                libsql::params![fetch_limit as i64],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?
        };

        let mut links = Vec::new();
        let mut has_more = false;

        let mut count = 0;
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            if count < limit_val {
                links.push(self.row_to_link(row)?);
                count += 1;
            } else {
                has_more = true;
                break;
            }
        }

        let next_cursor = if has_more && !links.is_empty() {
            use crate::commands::common::cursor;
            Some(cursor::encode(&links.last().unwrap().id))
        } else {
            None
        };

        Ok((links, next_cursor, has_more))
    }

    /// Helper to convert database row to ResourceLink
    fn row_to_link(&self, row: libsql::Row) -> Result<ResourceLink> {
        let id: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
        let source_type: String = row.get(1).map_err(|e| AppError::LibSQL(e))?;
        let source_id: String = row.get(2).map_err(|e| AppError::LibSQL(e))?;
        let target_type: String = row.get(3).map_err(|e| AppError::LibSQL(e))?;
        let target_id: String = row.get(4).map_err(|e| AppError::LibSQL(e))?;
        let link_text: Option<String> = row.get(5).map_err(|e| AppError::LibSQL(e))?;
        let created_at_str: String = row.get(6).map_err(|e| AppError::LibSQL(e))?;

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&Utc);

        Ok(ResourceLink {
            id,
            source_type,
            source_id,
            target_type,
            target_id,
            link_text,
            created_at,
        })
    }
}
