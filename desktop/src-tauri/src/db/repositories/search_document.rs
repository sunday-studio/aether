use crate::error::{AppError, Result};
use crate::utils::search_text::{
    extract_text_from_lexical_document, first_search_line, normalize_search_text, truncate_preview,
};
use chrono::Utc;
use libsql::Database;
use sha2::{Digest, Sha256};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SearchDocumentInput {
    pub resource_type: String,
    pub resource_id: String,
    pub chunk_index: i64,
    pub title: String,
    pub text: String,
    pub source_updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexStatus {
    pub total_documents: i64,
    pub entries: i64,
    pub tasks: i64,
    pub goals: i64,
    pub tags: i64,
    pub bookmarks: i64,
}

pub struct SearchDocumentRepository {
    database: Arc<Database>,
}

impl SearchDocumentRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    pub async fn reindex_all(&self) -> Result<SearchIndexStatus> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        conn.execute("DELETE FROM search_documents", libsql::params![])
            .await
            .map_err(AppError::LibSQL)?;

        self.reindex_entries().await?;
        self.reindex_tasks().await?;
        self.reindex_goals().await?;
        self.reindex_tags().await?;
        self.reindex_bookmarks().await?;
        self.status().await
    }

    pub async fn status(&self) -> Result<SearchIndexStatus> {
        Ok(SearchIndexStatus {
            total_documents: self.count_for_type(None).await?,
            entries: self.count_for_type(Some("entry")).await?,
            tasks: self.count_for_type(Some("task")).await?,
            goals: self.count_for_type(Some("goal")).await?,
            tags: self.count_for_type(Some("tag")).await?,
            bookmarks: self.count_for_type(Some("bookmark")).await?,
        })
    }

    pub async fn upsert_document(&self, input: SearchDocumentInput) -> Result<()> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let now = Utc::now().to_rfc3339();
        let text = normalize_search_text(&input.text);
        let title = normalize_search_text(&input.title);
        let hash = text_hash(&text);
        let id = format!(
            "{}:{}:{}",
            input.resource_type, input.resource_id, input.chunk_index
        );

        conn.execute(
            "INSERT INTO search_documents (
                id, resource_type, resource_id, chunk_index, title, text, text_hash,
                source_updated_at, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(resource_type, resource_id, chunk_index) DO UPDATE SET
                title = excluded.title,
                text = excluded.text,
                text_hash = excluded.text_hash,
                source_updated_at = excluded.source_updated_at,
                updated_at = excluded.updated_at",
            libsql::params![
                id,
                input.resource_type,
                input.resource_id,
                input.chunk_index,
                title,
                text,
                hash,
                input.source_updated_at,
                now.clone(),
                now,
            ],
        )
        .await
        .map_err(AppError::LibSQL)?;

        Ok(())
    }

    pub async fn delete_resource(&self, resource_type: &str, resource_id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        conn.execute(
            "DELETE FROM search_documents WHERE resource_type = ?1 AND resource_id = ?2",
            libsql::params![resource_type, resource_id],
        )
        .await
        .map_err(AppError::LibSQL)?;
        Ok(())
    }

    async fn reindex_entries(&self) -> Result<()> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT id, document, created_at, updated_at
                 FROM entries
                 WHERE is_deleted = 0 AND deleted_at IS NULL",
                libsql::params![],
            )
            .await
            .map_err(AppError::LibSQL)?;

        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            let id: String = row.get(0).map_err(AppError::LibSQL)?;
            let document: String = row.get(1).map_err(AppError::LibSQL)?;
            let created_at: String = row.get(2).map_err(AppError::LibSQL)?;
            let updated_at: String = row.get(3).map_err(AppError::LibSQL)?;
            let text = extract_text_from_lexical_document(&document).unwrap_or_default();
            let title = first_search_line(&text);

            self.upsert_document(SearchDocumentInput {
                resource_type: "entry".to_string(),
                resource_id: id,
                chunk_index: 0,
                title: if title.is_empty() {
                    truncate_preview(&text, 80)
                } else {
                    title
                },
                text,
                source_updated_at: if updated_at.is_empty() {
                    created_at
                } else {
                    updated_at
                },
            })
            .await?;
        }

        Ok(())
    }

    async fn reindex_tasks(&self) -> Result<()> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT id, title, description, updated_at
                 FROM tasks
                 WHERE deleted_at IS NULL",
                libsql::params![],
            )
            .await
            .map_err(AppError::LibSQL)?;

        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            let id: String = row.get(0).map_err(AppError::LibSQL)?;
            let title: String = row.get(1).map_err(AppError::LibSQL)?;
            let description: Option<String> = row.get(2).map_err(AppError::LibSQL)?;
            let updated_at: String = row.get(3).map_err(AppError::LibSQL)?;
            let text = [title.as_str(), description.as_deref().unwrap_or("")]
                .join(" ");

            self.upsert_document(SearchDocumentInput {
                resource_type: "task".to_string(),
                resource_id: id,
                chunk_index: 0,
                title,
                text,
                source_updated_at: updated_at,
            })
            .await?;
        }

        Ok(())
    }

    async fn reindex_goals(&self) -> Result<()> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT id, name, description, updated_at
                 FROM goals
                 WHERE deleted_at IS NULL",
                libsql::params![],
            )
            .await
            .map_err(AppError::LibSQL)?;

        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            let id: String = row.get(0).map_err(AppError::LibSQL)?;
            let name: String = row.get(1).map_err(AppError::LibSQL)?;
            let description: Option<String> = row.get(2).map_err(AppError::LibSQL)?;
            let updated_at: String = row.get(3).map_err(AppError::LibSQL)?;
            let text = [name.as_str(), description.as_deref().unwrap_or("")]
                .join(" ");

            self.upsert_document(SearchDocumentInput {
                resource_type: "goal".to_string(),
                resource_id: id,
                chunk_index: 0,
                title: name,
                text,
                source_updated_at: updated_at,
            })
            .await?;
        }

        Ok(())
    }

    async fn reindex_tags(&self) -> Result<()> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT id, name, updated_at
                 FROM tags
                 WHERE deleted_at IS NULL",
                libsql::params![],
            )
            .await
            .map_err(AppError::LibSQL)?;

        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            let id: String = row.get(0).map_err(AppError::LibSQL)?;
            let name: String = row.get(1).map_err(AppError::LibSQL)?;
            let updated_at: String = row.get(2).map_err(AppError::LibSQL)?;

            self.upsert_document(SearchDocumentInput {
                resource_type: "tag".to_string(),
                resource_id: id,
                chunk_index: 0,
                title: name.clone(),
                text: name,
                source_updated_at: updated_at,
            })
            .await?;
        }

        Ok(())
    }

    async fn reindex_bookmarks(&self) -> Result<()> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT id, url, title, description, site_name, author, updated_at
                 FROM bookmarks
                 WHERE is_deleted = 0 AND deleted_at IS NULL",
                libsql::params![],
            )
            .await
            .map_err(AppError::LibSQL)?;

        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            let id: String = row.get(0).map_err(AppError::LibSQL)?;
            let url: String = row.get(1).map_err(AppError::LibSQL)?;
            let title: Option<String> = row.get(2).map_err(AppError::LibSQL)?;
            let description: Option<String> = row.get(3).map_err(AppError::LibSQL)?;
            let site_name: Option<String> = row.get(4).map_err(AppError::LibSQL)?;
            let author: Option<String> = row.get(5).map_err(AppError::LibSQL)?;
            let updated_at: String = row.get(6).map_err(AppError::LibSQL)?;
            let title = title.unwrap_or_else(|| url.clone());
            let text = [
                title.as_str(),
                description.as_deref().unwrap_or(""),
                site_name.as_deref().unwrap_or(""),
                author.as_deref().unwrap_or(""),
                url.as_str(),
            ]
            .join(" ");

            self.upsert_document(SearchDocumentInput {
                resource_type: "bookmark".to_string(),
                resource_id: id,
                chunk_index: 0,
                title,
                text,
                source_updated_at: updated_at,
            })
            .await?;
        }

        Ok(())
    }

    async fn count_for_type(&self, resource_type: Option<&str>) -> Result<i64> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = if let Some(resource_type) = resource_type {
            conn.query(
                "SELECT COUNT(*) FROM search_documents WHERE resource_type = ?1",
                libsql::params![resource_type],
            )
            .await
            .map_err(AppError::LibSQL)?
        } else {
            conn.query("SELECT COUNT(*) FROM search_documents", libsql::params![])
                .await
                .map_err(AppError::LibSQL)?
        };

        if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            Ok(row.get(0).map_err(AppError::LibSQL)?)
        } else {
            Ok(0)
        }
    }
}

fn text_hash(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}
