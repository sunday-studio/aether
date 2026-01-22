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
        
        let id = generate_id("link");
        let now = Utc::now();
        let created_at_str = now.to_rfc3339();

        conn.execute(
            "INSERT INTO resource_links (id, source_type, source_id, target_type, target_id, link_text, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            libsql::params![
                id.clone(),
                source_type.clone(),
                source_id.clone(),
                target_type.clone(),
                target_id.clone(),
                link_text.as_ref().map(|s| s.as_str()),
                created_at_str
            ],
        )
        .await
        .map_err(|e| {
            // Check if it's a unique constraint violation
            if e.to_string().contains("UNIQUE constraint") {
                AppError::BadRequest(format!(
                    "Link already exists from {}:{} to {}:{}",
                    source_type, source_id, target_type, target_id
                ))
            } else {
                AppError::LibSQL(e)
            }
        })?;

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
                 WHERE source_type = ?1 AND source_id = ?2 
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
                 WHERE target_type = ?1 AND target_id = ?2 
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
        
        conn.execute(
            "DELETE FROM resource_links 
             WHERE source_type = ?1 AND source_id = ?2 AND target_type = ?3 AND target_id = ?4",
            libsql::params![source_type, source_id, target_type, target_id],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Delete all links from a source resource (when resource is deleted)
    pub async fn delete_by_source(
        &self,
        source_type: &str,
        source_id: &str,
    ) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        conn.execute(
            "DELETE FROM resource_links WHERE source_type = ?1 AND source_id = ?2",
            libsql::params![source_type, source_id],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Delete all links to a target resource (when resource is deleted)
    pub async fn delete_by_target(
        &self,
        target_type: &str,
        target_id: &str,
    ) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        conn.execute(
            "DELETE FROM resource_links WHERE target_type = ?1 AND target_id = ?2",
            libsql::params![target_type, target_id],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Get all links for graph visualization
    pub async fn find_all_for_graph(&self) -> Result<Vec<ResourceLink>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT id, source_type, source_id, target_type, target_id, link_text, created_at 
                 FROM resource_links 
                 ORDER BY created_at DESC",
                libsql::params![],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut links = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            links.push(self.row_to_link(row)?);
        }

        Ok(links)
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
