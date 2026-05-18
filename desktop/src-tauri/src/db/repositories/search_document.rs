use crate::error::{AppError, Result};
use crate::utils::embeddings::generate_embedding;
use crate::utils::search_text::{
    extract_text_from_lexical_document, first_search_line, normalize_search_text, truncate_preview,
};
use chrono::Utc;
use libsql::Database;
use sha2::{Digest, Sha256};
use std::sync::Arc;

const DEFAULT_SEMANTIC_MODEL: &str = "all-MiniLM-L6-v2";

#[derive(Debug, Clone)]
pub struct SearchDocumentResult {
    pub id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub title: String,
    pub preview: String,
    pub score: f64,
    pub match_kind: String,
    pub highlights: Vec<String>,
    pub source_updated_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct SearchDocumentQuery {
    pub resource_types: Option<Vec<String>>,
    pub tag_ids: Option<Vec<String>>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SearchDocumentPage {
    pub results: Vec<SearchDocumentResult>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

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

#[derive(Debug, Clone)]
struct SearchDocumentSource {
    text: String,
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
        drop(conn);

        self.reindex_entries().await?;
        self.reindex_tasks().await?;
        self.reindex_goals().await?;
        self.reindex_tags().await?;
        self.reindex_bookmarks().await?;
        self.status().await
    }

    pub async fn reindex_resource(&self, resource_type: &str, resource_id: &str) -> Result<()> {
        match resource_type {
            "entry" => self.reindex_entry(resource_id).await,
            "task" => self.reindex_task(resource_id).await,
            "goal" => self.reindex_goal(resource_id).await,
            "tag" => self.reindex_tag(resource_id).await,
            "bookmark" => self.reindex_bookmark(resource_id).await,
            _ => Err(AppError::BadRequest(format!(
                "Unsupported search resource type: {}",
                resource_type
            ))),
        }
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

    pub async fn search_keyword(
        &self,
        query: &str,
        filters: SearchDocumentQuery,
    ) -> Result<SearchDocumentPage> {
        if query.trim().is_empty() {
            return Ok(SearchDocumentPage {
                results: Vec::new(),
                next_cursor: None,
                has_more: false,
            });
        }

        let limit = filters.limit.unwrap_or(50).min(100) as usize;
        let offset = search_offset(filters.cursor.as_deref(), filters.offset)?;
        let escaped_query = escape_fts_query(query);
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT
                    d.id,
                    d.resource_type,
                    d.resource_id,
                    d.title,
                    d.text,
                    d.source_updated_at,
                    d.created_at,
                    d.updated_at,
                    bm25(search_documents_fts) AS rank
                 FROM search_documents_fts
                 JOIN search_documents d ON d.rowid = search_documents_fts.rowid
                 WHERE search_documents_fts MATCH ?1
                 ORDER BY rank
                 LIMIT 500",
                libsql::params![escaped_query],
            )
            .await
            .map_err(AppError::LibSQL)?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            let resource_type: String = row.get(1).map_err(AppError::LibSQL)?;
            if !matches_resource_type(&filters.resource_types, &resource_type) {
                continue;
            }

            let source_updated_at: String = row.get(5).map_err(AppError::LibSQL)?;
            if !matches_date_range(
                &source_updated_at,
                filters.date_from.as_deref(),
                filters.date_to.as_deref(),
            ) {
                continue;
            }

            let resource_id: String = row.get(2).map_err(AppError::LibSQL)?;
            if !self
                .matches_tags(&conn, &resource_type, &resource_id, &filters.tag_ids)
                .await?
            {
                continue;
            }

            let text: String = row.get(4).map_err(AppError::LibSQL)?;
            let rank: f64 = row.get(8).unwrap_or(0.0);
            let score = if rank < 0.0 {
                1.0 / (1.0 + rank.abs())
            } else {
                1.0 / (1.0 + rank)
            };

            let mut result = SearchDocumentResult {
                id: row.get(0).map_err(AppError::LibSQL)?,
                resource_type,
                resource_id,
                title: row.get(3).map_err(AppError::LibSQL)?,
                preview: truncate_preview(&text, 220),
                score,
                match_kind: "keyword".to_string(),
                highlights: Vec::new(),
                source_updated_at,
                created_at: row.get(6).map_err(AppError::LibSQL)?,
                updated_at: row.get(7).map_err(AppError::LibSQL)?,
            };
            self.apply_ranking_boosts(&conn, query, &mut result).await?;
            results.push(result);
        }

        let mut page_results: Vec<_> = results.into_iter().skip(offset).take(limit + 1).collect();
        let has_more = page_results.len() > limit;
        if has_more {
            page_results.truncate(limit);
        }
        let next_cursor = if has_more {
            Some(crate::commands::common::cursor::encode(
                &(offset + page_results.len()).to_string(),
            ))
        } else {
            None
        };

        Ok(SearchDocumentPage {
            results: page_results,
            next_cursor,
            has_more,
        })
    }

    pub async fn search_semantic(
        &self,
        query: &str,
        filters: SearchDocumentQuery,
    ) -> Result<SearchDocumentPage> {
        if query.trim().is_empty() {
            return Ok(empty_search_document_page());
        }

        let limit = filters.limit.unwrap_or(50).min(100) as usize;
        let offset = search_offset(filters.cursor.as_deref(), filters.offset)?;
        let query_embedding = generate_embedding(query).await?;
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT
                    d.id,
                    d.resource_type,
                    d.resource_id,
                    d.title,
                    d.text,
                    d.source_updated_at,
                    d.created_at,
                    d.updated_at,
                    e.vector,
                    e.dimensions
                 FROM search_embeddings e
                 JOIN search_documents d ON d.id = e.search_document_id
                 WHERE e.model_name = ?1
                 LIMIT 1000",
                libsql::params![DEFAULT_SEMANTIC_MODEL],
            )
            .await
            .map_err(AppError::LibSQL)?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            let resource_type: String = row.get(1).map_err(AppError::LibSQL)?;
            if !matches_resource_type(&filters.resource_types, &resource_type) {
                continue;
            }

            let source_updated_at: String = row.get(5).map_err(AppError::LibSQL)?;
            if !matches_date_range(
                &source_updated_at,
                filters.date_from.as_deref(),
                filters.date_to.as_deref(),
            ) {
                continue;
            }

            let resource_id: String = row.get(2).map_err(AppError::LibSQL)?;
            if !self
                .matches_tags(&conn, &resource_type, &resource_id, &filters.tag_ids)
                .await?
            {
                continue;
            }

            let dimensions: i64 = row.get(9).map_err(AppError::LibSQL)?;
            let vector_bytes: Vec<u8> = row.get(8).map_err(AppError::LibSQL)?;
            let vector = decode_vector(&vector_bytes, dimensions)?;
            if vector.len() != query_embedding.len() {
                continue;
            }

            let text: String = row.get(4).map_err(AppError::LibSQL)?;
            let score = cosine_similarity(&query_embedding, &vector);

            let mut result = SearchDocumentResult {
                id: row.get(0).map_err(AppError::LibSQL)?,
                resource_type,
                resource_id,
                title: row.get(3).map_err(AppError::LibSQL)?,
                preview: truncate_preview(&text, 220),
                score,
                match_kind: "semantic".to_string(),
                highlights: Vec::new(),
                source_updated_at,
                created_at: row.get(6).map_err(AppError::LibSQL)?,
                updated_at: row.get(7).map_err(AppError::LibSQL)?,
            };
            self.apply_ranking_boosts(&conn, query, &mut result).await?;
            results.push(result);
        }

        results.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        paged_results(results, offset, limit)
    }

    pub async fn search_hybrid(
        &self,
        query: &str,
        filters: SearchDocumentQuery,
    ) -> Result<SearchDocumentPage> {
        let limit = filters.limit.unwrap_or(50).min(100) as usize;
        let offset = search_offset(filters.cursor.as_deref(), filters.offset)?;
        let mut keyword_filters = filters.clone();
        keyword_filters.limit = Some(100);
        keyword_filters.offset = None;
        keyword_filters.cursor = None;

        let semantic_filters = keyword_filters.clone();
        let keyword_page = self.search_keyword(query, keyword_filters).await?;
        let semantic_page = self.search_semantic(query, semantic_filters).await?;
        let mut merged = std::collections::HashMap::<String, SearchDocumentResult>::new();

        for mut result in keyword_page.results {
            result.score *= 0.65;
            result.match_kind = "hybrid".to_string();
            merged.insert(result.id.clone(), result);
        }

        for result in semantic_page.results {
            let semantic_score = result.score * 0.35;
            merged
                .entry(result.id.clone())
                .and_modify(|existing| {
                    existing.score += semantic_score;
                })
                .or_insert_with(|| SearchDocumentResult {
                    score: semantic_score,
                    match_kind: "hybrid".to_string(),
                    ..result
                });
        }

        let mut results = merged.into_values().collect::<Vec<_>>();
        results.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        paged_results(results, offset, limit)
    }

    pub async fn find_related(
        &self,
        resource_type: &str,
        resource_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<SearchDocumentResult>> {
        let Some(source) = self
            .load_resource_document(resource_type, resource_id)
            .await?
        else {
            return Ok(Vec::new());
        };
        let query = related_query_terms(&source.text);
        if query.is_empty() {
            return Ok(Vec::new());
        }

        let page = self
            .search_keyword(
                &query,
                SearchDocumentQuery {
                    limit: Some(limit.unwrap_or(10).min(50) + 1),
                    ..SearchDocumentQuery::default()
                },
            )
            .await?;

        Ok(page
            .results
            .into_iter()
            .filter(|result| {
                result.resource_type != resource_type || result.resource_id != resource_id
            })
            .take(limit.unwrap_or(10).min(50) as usize)
            .collect())
    }

    pub async fn list_context_by_date_range(
        &self,
        date_from: &str,
        date_to: &str,
        limit: Option<u32>,
    ) -> Result<Vec<SearchDocumentResult>> {
        let limit = limit.unwrap_or(50).min(100);
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT id, resource_type, resource_id, title, text, source_updated_at, created_at, updated_at
                 FROM search_documents
                 WHERE source_updated_at >= ?1 AND source_updated_at <= ?2
                 ORDER BY source_updated_at DESC, id DESC
                 LIMIT ?3",
                libsql::params![date_from, date_to, limit],
            )
            .await
            .map_err(AppError::LibSQL)?;
        let mut results = Vec::new();

        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            let text: String = row.get(4).map_err(AppError::LibSQL)?;
            results.push(SearchDocumentResult {
                id: row.get(0).map_err(AppError::LibSQL)?,
                resource_type: row.get(1).map_err(AppError::LibSQL)?,
                resource_id: row.get(2).map_err(AppError::LibSQL)?,
                title: row.get(3).map_err(AppError::LibSQL)?,
                preview: truncate_preview(&text, 220),
                score: 1.0,
                match_kind: "date".to_string(),
                highlights: Vec::new(),
                source_updated_at: row.get(5).map_err(AppError::LibSQL)?,
                created_at: row.get(6).map_err(AppError::LibSQL)?,
                updated_at: row.get(7).map_err(AppError::LibSQL)?,
            });
        }

        Ok(results)
    }

    async fn load_resource_document(
        &self,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<Option<SearchDocumentSource>> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT text
                 FROM search_documents
                 WHERE resource_type = ?1 AND resource_id = ?2
                 ORDER BY chunk_index
                 LIMIT 1",
                libsql::params![resource_type, resource_id],
            )
            .await
            .map_err(AppError::LibSQL)?;

        let Some(row) = rows.next().await.map_err(AppError::LibSQL)? else {
            return Ok(None);
        };

        Ok(Some(SearchDocumentSource {
            text: row.get(0).map_err(AppError::LibSQL)?,
        }))
    }

    async fn matches_tags(
        &self,
        conn: &libsql::Connection,
        resource_type: &str,
        resource_id: &str,
        tag_ids: &Option<Vec<String>>,
    ) -> Result<bool> {
        let Some(tag_ids) = tag_ids else {
            return Ok(true);
        };
        if tag_ids.is_empty() {
            return Ok(true);
        }
        if resource_type == "tag" {
            return Ok(tag_ids.iter().any(|tag_id| tag_id == resource_id));
        }

        let table = match resource_type {
            "entry" => "entry_tags",
            "task" => "task_tags",
            "goal" => "goal_tags",
            "bookmark" => "bookmark_tags",
            _ => return Ok(false),
        };
        let id_column = match resource_type {
            "entry" => "entry_id",
            "task" => "task_id",
            "goal" => "goal_id",
            "bookmark" => "bookmark_id",
            _ => return Ok(false),
        };

        for tag_id in tag_ids {
            let query = format!(
                "SELECT 1 FROM {} WHERE {} = ?1 AND tag_id = ?2 LIMIT 1",
                table, id_column
            );
            let mut rows = conn
                .query(&query, libsql::params![resource_id, tag_id.as_str()])
                .await
                .map_err(AppError::LibSQL)?;
            if rows.next().await.map_err(AppError::LibSQL)?.is_some() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn apply_ranking_boosts(
        &self,
        conn: &libsql::Connection,
        query: &str,
        result: &mut SearchDocumentResult,
    ) -> Result<()> {
        let normalized_query = normalize_search_text(query).to_lowercase();
        let normalized_title = result.title.to_lowercase();

        if !normalized_query.is_empty() && normalized_title == normalized_query {
            result.score += 0.25;
        } else if !normalized_query.is_empty() && normalized_title.contains(&normalized_query) {
            result.score += 0.12;
        }

        if self
            .resource_has_matching_tag(
                conn,
                &result.resource_type,
                &result.resource_id,
                &normalized_query,
            )
            .await?
        {
            result.score += 0.08;
        }

        match result.resource_type.as_str() {
            "entry" if self.entry_is_pinned(conn, &result.resource_id).await? => {
                result.score += 0.05;
            }
            "task" if self.task_is_incomplete(conn, &result.resource_id).await? => {
                result.score += 0.05;
            }
            "goal"
                if self
                    .goal_has_current_instance(conn, &result.resource_id)
                    .await? =>
            {
                result.score += 0.05;
            }
            _ => {}
        }

        result.score = result.score.min(1.0);
        Ok(())
    }

    async fn resource_has_matching_tag(
        &self,
        conn: &libsql::Connection,
        resource_type: &str,
        resource_id: &str,
        normalized_query: &str,
    ) -> Result<bool> {
        if normalized_query.is_empty() {
            return Ok(false);
        }

        let (table, id_column) = match resource_type {
            "entry" => ("entry_tags", "entry_id"),
            "task" => ("task_tags", "task_id"),
            "goal" => ("goal_tags", "goal_id"),
            "bookmark" => ("bookmark_tags", "bookmark_id"),
            "tag" => return Ok(false),
            _ => return Ok(false),
        };
        let query = format!(
            "SELECT t.name
             FROM tags t
             JOIN {} rt ON rt.tag_id = t.id
             WHERE rt.{} = ?1 AND t.deleted_at IS NULL",
            table, id_column
        );
        let mut rows = conn
            .query(&query, libsql::params![resource_id])
            .await
            .map_err(AppError::LibSQL)?;

        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            let tag_name: String = row.get(0).map_err(AppError::LibSQL)?;
            let normalized_tag = tag_name.to_lowercase();
            if normalized_tag == normalized_query || normalized_query.contains(&normalized_tag) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn entry_is_pinned(&self, conn: &libsql::Connection, resource_id: &str) -> Result<bool> {
        let mut rows = conn
            .query(
                "SELECT is_pinned FROM entries WHERE id = ?1 AND deleted_at IS NULL LIMIT 1",
                libsql::params![resource_id],
            )
            .await
            .map_err(AppError::LibSQL)?;

        Ok(rows
            .next()
            .await
            .map_err(AppError::LibSQL)?
            .map(|row| row.get::<i64>(0).unwrap_or(0) != 0)
            .unwrap_or(false))
    }

    async fn task_is_incomplete(
        &self,
        conn: &libsql::Connection,
        resource_id: &str,
    ) -> Result<bool> {
        let mut rows = conn
            .query(
                "SELECT is_completed FROM tasks WHERE id = ?1 AND deleted_at IS NULL LIMIT 1",
                libsql::params![resource_id],
            )
            .await
            .map_err(AppError::LibSQL)?;

        Ok(rows
            .next()
            .await
            .map_err(AppError::LibSQL)?
            .map(|row| row.get::<i64>(0).unwrap_or(1) == 0)
            .unwrap_or(false))
    }

    async fn goal_has_current_instance(
        &self,
        conn: &libsql::Connection,
        resource_id: &str,
    ) -> Result<bool> {
        let now = Utc::now().to_rfc3339();
        let mut rows = conn
            .query(
                "SELECT 1
                 FROM goal_instances
                 WHERE goal_id = ?1
                   AND deleted_at IS NULL
                   AND status = 'active'
                   AND period_start <= ?2
                   AND (period_end IS NULL OR period_end >= ?2)
                 LIMIT 1",
                libsql::params![resource_id, now],
            )
            .await
            .map_err(AppError::LibSQL)?;

        Ok(rows.next().await.map_err(AppError::LibSQL)?.is_some())
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

    async fn reindex_entry(&self, resource_id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT id, document, created_at, updated_at
                 FROM entries
                 WHERE id = ?1 AND is_deleted = 0 AND deleted_at IS NULL",
                libsql::params![resource_id],
            )
            .await
            .map_err(AppError::LibSQL)?;

        let input = if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            Some(Self::entry_input_from_row(row)?)
        } else {
            None
        };
        drop(rows);
        drop(conn);

        let Some(input) = input else {
            return self.delete_resource("entry", resource_id).await;
        };

        self.upsert_document(input).await
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

        let mut inputs = Vec::new();
        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            inputs.push(Self::entry_input_from_row(row)?);
        }
        drop(rows);
        drop(conn);

        for input in inputs {
            self.upsert_document(input).await?;
        }

        Ok(())
    }

    fn entry_input_from_row(row: libsql::Row) -> Result<SearchDocumentInput> {
        let id: String = row.get(0).map_err(AppError::LibSQL)?;
        let document: String = row.get(1).map_err(AppError::LibSQL)?;
        let created_at: String = row.get(2).map_err(AppError::LibSQL)?;
        let updated_at: String = row.get(3).map_err(AppError::LibSQL)?;
        let text = extract_text_from_lexical_document(&document).unwrap_or_default();
        let title = first_search_line(&text);

        Ok(SearchDocumentInput {
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
    }

    async fn reindex_task(&self, resource_id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT id, title, description, updated_at
                 FROM tasks
                 WHERE id = ?1 AND deleted_at IS NULL",
                libsql::params![resource_id],
            )
            .await
            .map_err(AppError::LibSQL)?;

        let input = if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            Some(Self::task_input_from_row(row)?)
        } else {
            None
        };
        drop(rows);
        drop(conn);

        let Some(input) = input else {
            return self.delete_resource("task", resource_id).await;
        };

        self.upsert_document(input).await
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

        let mut inputs = Vec::new();
        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            inputs.push(Self::task_input_from_row(row)?);
        }
        drop(rows);
        drop(conn);

        for input in inputs {
            self.upsert_document(input).await?;
        }

        Ok(())
    }

    fn task_input_from_row(row: libsql::Row) -> Result<SearchDocumentInput> {
        let id: String = row.get(0).map_err(AppError::LibSQL)?;
        let title: String = row.get(1).map_err(AppError::LibSQL)?;
        let description: Option<String> = row.get(2).map_err(AppError::LibSQL)?;
        let updated_at: String = row.get(3).map_err(AppError::LibSQL)?;
        let text = [title.as_str(), description.as_deref().unwrap_or("")].join(" ");

        Ok(SearchDocumentInput {
            resource_type: "task".to_string(),
            resource_id: id,
            chunk_index: 0,
            title,
            text,
            source_updated_at: updated_at,
        })
    }

    async fn reindex_goal(&self, resource_id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT id, name, description, updated_at
                 FROM goals
                 WHERE id = ?1 AND deleted_at IS NULL",
                libsql::params![resource_id],
            )
            .await
            .map_err(AppError::LibSQL)?;

        let input = if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            Some(Self::goal_input_from_row(row)?)
        } else {
            None
        };
        drop(rows);
        drop(conn);

        let Some(input) = input else {
            return self.delete_resource("goal", resource_id).await;
        };

        self.upsert_document(input).await
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

        let mut inputs = Vec::new();
        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            inputs.push(Self::goal_input_from_row(row)?);
        }
        drop(rows);
        drop(conn);

        for input in inputs {
            self.upsert_document(input).await?;
        }

        Ok(())
    }

    fn goal_input_from_row(row: libsql::Row) -> Result<SearchDocumentInput> {
        let id: String = row.get(0).map_err(AppError::LibSQL)?;
        let name: String = row.get(1).map_err(AppError::LibSQL)?;
        let description: Option<String> = row.get(2).map_err(AppError::LibSQL)?;
        let updated_at: String = row.get(3).map_err(AppError::LibSQL)?;
        let text = [name.as_str(), description.as_deref().unwrap_or("")].join(" ");

        Ok(SearchDocumentInput {
            resource_type: "goal".to_string(),
            resource_id: id,
            chunk_index: 0,
            title: name,
            text,
            source_updated_at: updated_at,
        })
    }

    async fn reindex_tag(&self, resource_id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT id, name, updated_at
                 FROM tags
                 WHERE id = ?1 AND deleted_at IS NULL",
                libsql::params![resource_id],
            )
            .await
            .map_err(AppError::LibSQL)?;

        let input = if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            Some(Self::tag_input_from_row(row)?)
        } else {
            None
        };
        drop(rows);
        drop(conn);

        let Some(input) = input else {
            return self.delete_resource("tag", resource_id).await;
        };

        self.upsert_document(input).await
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

        let mut inputs = Vec::new();
        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            inputs.push(Self::tag_input_from_row(row)?);
        }
        drop(rows);
        drop(conn);

        for input in inputs {
            self.upsert_document(input).await?;
        }

        Ok(())
    }

    fn tag_input_from_row(row: libsql::Row) -> Result<SearchDocumentInput> {
        let id: String = row.get(0).map_err(AppError::LibSQL)?;
        let name: String = row.get(1).map_err(AppError::LibSQL)?;
        let updated_at: String = row.get(2).map_err(AppError::LibSQL)?;

        Ok(SearchDocumentInput {
            resource_type: "tag".to_string(),
            resource_id: id,
            chunk_index: 0,
            title: name.clone(),
            text: name,
            source_updated_at: updated_at,
        })
    }

    async fn reindex_bookmark(&self, resource_id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(AppError::LibSQL)?;
        let mut rows = conn
            .query(
                "SELECT id, url, title, description, site_name, author, updated_at
                 FROM bookmarks
                 WHERE id = ?1 AND is_deleted = 0 AND deleted_at IS NULL",
                libsql::params![resource_id],
            )
            .await
            .map_err(AppError::LibSQL)?;

        let input = if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            Some(Self::bookmark_input_from_row(row)?)
        } else {
            None
        };
        drop(rows);
        drop(conn);

        let Some(input) = input else {
            return self.delete_resource("bookmark", resource_id).await;
        };

        self.upsert_document(input).await
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

        let mut inputs = Vec::new();
        while let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
            inputs.push(Self::bookmark_input_from_row(row)?);
        }
        drop(rows);
        drop(conn);

        for input in inputs {
            self.upsert_document(input).await?;
        }

        Ok(())
    }

    fn bookmark_input_from_row(row: libsql::Row) -> Result<SearchDocumentInput> {
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

        Ok(SearchDocumentInput {
            resource_type: "bookmark".to_string(),
            resource_id: id,
            chunk_index: 0,
            title,
            text,
            source_updated_at: updated_at,
        })
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

fn escape_fts_query(query: &str) -> String {
    query.replace('"', "\"\"")
}

fn matches_resource_type(resource_types: &Option<Vec<String>>, resource_type: &str) -> bool {
    resource_types
        .as_ref()
        .map(|types| types.iter().any(|item| item == resource_type))
        .unwrap_or(true)
}

fn matches_date_range(value: &str, date_from: Option<&str>, date_to: Option<&str>) -> bool {
    if let Some(date_from) = date_from {
        if value < date_from {
            return false;
        }
    }

    if let Some(date_to) = date_to {
        if value > date_to {
            return false;
        }
    }

    true
}

fn search_offset(cursor: Option<&str>, offset: Option<u32>) -> Result<usize> {
    let Some(cursor) = cursor else {
        return Ok(offset.unwrap_or(0) as usize);
    };

    let decoded = crate::commands::common::cursor::decode(cursor)?;
    decoded.parse::<usize>().map_err(|e| {
        AppError::BadRequest(format!("Invalid search cursor offset '{}': {}", decoded, e))
    })
}

fn empty_search_document_page() -> SearchDocumentPage {
    SearchDocumentPage {
        results: Vec::new(),
        next_cursor: None,
        has_more: false,
    }
}

fn paged_results(
    results: Vec<SearchDocumentResult>,
    offset: usize,
    limit: usize,
) -> Result<SearchDocumentPage> {
    let mut page_results: Vec<_> = results.into_iter().skip(offset).take(limit + 1).collect();
    let has_more = page_results.len() > limit;
    if has_more {
        page_results.truncate(limit);
    }
    let next_cursor = if has_more {
        Some(crate::commands::common::cursor::encode(
            &(offset + page_results.len()).to_string(),
        ))
    } else {
        None
    };

    Ok(SearchDocumentPage {
        results: page_results,
        next_cursor,
        has_more,
    })
}

fn decode_vector(bytes: &[u8], dimensions: i64) -> Result<Vec<f32>> {
    let expected_len = dimensions as usize * std::mem::size_of::<f32>();
    if bytes.len() != expected_len {
        return Err(AppError::Internal(format!(
            "Stored embedding byte length {} does not match dimensions {}",
            bytes.len(),
            dimensions
        )));
    }

    Ok(bytes
        .chunks_exact(std::mem::size_of::<f32>())
        .map(|chunk| {
            let mut value = [0u8; 4];
            value.copy_from_slice(chunk);
            f32::from_le_bytes(value)
        })
        .collect())
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> f64 {
    let mut dot = 0.0f64;
    let mut left_norm = 0.0f64;
    let mut right_norm = 0.0f64;

    for (left_value, right_value) in left.iter().zip(right.iter()) {
        let left_value = *left_value as f64;
        let right_value = *right_value as f64;
        dot += left_value * right_value;
        left_norm += left_value * left_value;
        right_norm += right_value * right_value;
    }

    if left_norm == 0.0 || right_norm == 0.0 {
        return 0.0;
    }

    ((dot / (left_norm.sqrt() * right_norm.sqrt())) + 1.0) / 2.0
}

fn related_query_terms(text: &str) -> String {
    text.split_whitespace()
        .map(|term| {
            term.trim_matches(|ch: char| !ch.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|term| term.len() >= 4)
        .take(6)
        .collect::<Vec<_>>()
        .join(" OR ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::db::repositories::SearchEmbeddingRepository;
    use libsql::Builder;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static DB_COUNTER: AtomicU64 = AtomicU64::new(0);

    async fn test_repo() -> (Arc<Database>, SearchDocumentRepository, PathBuf) {
        let id = DB_COUNTER.fetch_add(1, Ordering::Relaxed);
        let db_path = std::env::temp_dir().join(format!(
            "aether-search-test-{}-{}.db",
            std::process::id(),
            id
        ));
        let database = Builder::new_local(&db_path)
            .build()
            .await
            .expect("create test database");
        migrations::run_migrations(&database)
            .await
            .expect("run migrations");
        let database = Arc::new(database);
        let repo = SearchDocumentRepository::new(database.clone());
        (database, repo, db_path)
    }

    async fn seed_search_resources(database: &Database) {
        let conn = database.connect().expect("connect to test database");
        let entry_document = r#"{"root":{"children":[{"children":[{"text":"Morning clarity journal entry","type":"text"}],"type":"paragraph"}],"type":"root"}}"#;

        conn.execute(
            "INSERT INTO entries (id, document, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            libsql::params![
                "entry-1",
                entry_document,
                "2026-05-10T09:00:00Z",
                "2026-05-11T09:00:00Z"
            ],
        )
        .await
        .expect("seed entry");

        conn.execute(
            "INSERT INTO tasks (id, title, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            libsql::params![
                "task-1",
                "Plan search testing",
                "Cover repository indexing",
                "2026-05-12T09:00:00Z",
                "2026-05-12T09:00:00Z"
            ],
        )
        .await
        .expect("seed task");

        conn.execute(
            "INSERT INTO goals (id, name, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            libsql::params![
                "goal-1",
                "Improve recall",
                "Search across local notes",
                "2026-05-13T09:00:00Z",
                "2026-05-13T09:00:00Z"
            ],
        )
        .await
        .expect("seed goal");

        conn.execute(
            "INSERT INTO tags (id, name, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            libsql::params![
                "tag-1",
                "reflection",
                "2026-05-14T09:00:00Z",
                "2026-05-14T09:00:00Z"
            ],
        )
        .await
        .expect("seed tag");

        conn.execute(
            "INSERT INTO bookmarks (id, url, title, description, site_name, author, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            libsql::params![
                "bookmark-1",
                "https://example.com/search",
                "Search reference",
                "Useful retrieval notes",
                "Example",
                "Aether",
                "2026-05-15T09:00:00Z",
                "2026-05-15T09:00:00Z"
            ],
        )
        .await
        .expect("seed bookmark");

        conn.execute(
            "INSERT INTO bookmark_tags (bookmark_id, tag_id) VALUES (?1, ?2)",
            libsql::params!["bookmark-1", "tag-1"],
        )
        .await
        .expect("seed bookmark tag");
    }

    fn cleanup_db(path: PathBuf) {
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
    }

    #[tokio::test]
    async fn reindex_all_indexes_supported_resources() {
        let (database, repo, db_path) = test_repo().await;
        seed_search_resources(&database).await;

        let status = repo.reindex_all().await.expect("reindex all resources");

        assert_eq!(status.total_documents, 5);
        assert_eq!(status.entries, 1);
        assert_eq!(status.tasks, 1);
        assert_eq!(status.goals, 1);
        assert_eq!(status.tags, 1);
        assert_eq!(status.bookmarks, 1);

        cleanup_db(db_path);
    }

    #[tokio::test]
    async fn reindex_resource_removes_deleted_source_rows() {
        let (database, repo, db_path) = test_repo().await;
        seed_search_resources(&database).await;
        repo.reindex_all().await.expect("reindex all resources");

        let conn = database.connect().expect("connect to test database");
        conn.execute(
            "UPDATE entries SET is_deleted = 1, deleted_at = ?1 WHERE id = ?2",
            libsql::params!["2026-05-16T09:00:00Z", "entry-1"],
        )
        .await
        .expect("soft delete entry");

        repo.reindex_resource("entry", "entry-1")
            .await
            .expect("reindex deleted entry");
        let status = repo.status().await.expect("read entry status");

        assert_eq!(status.total_documents, 4);
        assert_eq!(status.entries, 0);

        conn.execute(
            "UPDATE tasks SET deleted_at = ?1 WHERE id = ?2",
            libsql::params!["2026-05-16T09:00:00Z", "task-1"],
        )
        .await
        .expect("soft delete task");

        repo.reindex_resource("task", "task-1")
            .await
            .expect("reindex deleted task");
        let status = repo.status().await.expect("read task status");

        assert_eq!(status.total_documents, 3);
        assert_eq!(status.tasks, 0);

        conn.execute(
            "UPDATE goals SET deleted_at = ?1 WHERE id = ?2",
            libsql::params!["2026-05-16T09:00:00Z", "goal-1"],
        )
        .await
        .expect("soft delete goal");

        repo.reindex_resource("goal", "goal-1")
            .await
            .expect("reindex deleted goal");
        let status = repo.status().await.expect("read goal status");

        assert_eq!(status.total_documents, 2);
        assert_eq!(status.goals, 0);

        conn.execute(
            "UPDATE bookmarks SET is_deleted = 1, deleted_at = ?1 WHERE id = ?2",
            libsql::params!["2026-05-16T09:00:00Z", "bookmark-1"],
        )
        .await
        .expect("soft delete bookmark");

        repo.reindex_resource("bookmark", "bookmark-1")
            .await
            .expect("reindex deleted bookmark");
        let status = repo.status().await.expect("read bookmark status");

        assert_eq!(status.total_documents, 1);
        assert_eq!(status.bookmarks, 0);

        cleanup_db(db_path);
    }

    #[tokio::test]
    async fn delete_resource_removes_indexed_documents() {
        let (database, repo, db_path) = test_repo().await;
        seed_search_resources(&database).await;
        repo.reindex_all().await.expect("reindex all resources");

        repo.delete_resource("bookmark", "bookmark-1")
            .await
            .expect("delete bookmark search document");
        let status = repo.status().await.expect("read status");
        let results = repo
            .search_keyword("reference", SearchDocumentQuery::default())
            .await
            .expect("search after delete");

        assert_eq!(status.total_documents, 4);
        assert_eq!(status.bookmarks, 0);
        assert!(results
            .results
            .iter()
            .all(|result| result.resource_id != "bookmark-1"));

        cleanup_db(db_path);
    }

    #[tokio::test]
    async fn search_keyword_returns_normalized_filtered_results() {
        let (database, repo, db_path) = test_repo().await;
        seed_search_resources(&database).await;
        repo.reindex_all().await.expect("reindex all resources");

        let results = repo
            .search_keyword(
                "search",
                SearchDocumentQuery {
                    resource_types: Some(vec!["task".to_string(), "bookmark".to_string()]),
                    tag_ids: None,
                    date_from: Some("2026-05-12T00:00:00Z".to_string()),
                    date_to: Some("2026-05-15T23:59:59Z".to_string()),
                    limit: Some(1),
                    offset: Some(0),
                    cursor: None,
                },
            )
            .await
            .expect("search keyword");

        assert_eq!(results.results.len(), 1);
        assert!(matches!(
            results.results[0].resource_type.as_str(),
            "task" | "bookmark"
        ));
        assert_eq!(results.results[0].match_kind, "keyword");
        assert!(!results.results[0].resource_id.is_empty());
        assert!(!results.results[0].title.is_empty());
        assert!(!results.results[0].preview.is_empty());

        cleanup_db(db_path);
    }

    #[tokio::test]
    async fn search_keyword_filters_results_by_tag() {
        let (database, repo, db_path) = test_repo().await;
        seed_search_resources(&database).await;
        repo.reindex_all().await.expect("reindex all resources");

        let results = repo
            .search_keyword(
                "search",
                SearchDocumentQuery {
                    tag_ids: Some(vec!["tag-1".to_string()]),
                    ..SearchDocumentQuery::default()
                },
            )
            .await
            .expect("search keyword by tag");

        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].resource_type, "bookmark");
        assert_eq!(results.results[0].resource_id, "bookmark-1");

        cleanup_db(db_path);
    }

    #[tokio::test]
    async fn search_keyword_returns_cursor_for_next_page() {
        let (database, repo, db_path) = test_repo().await;
        seed_search_resources(&database).await;
        repo.reindex_all().await.expect("reindex all resources");

        let first_page = repo
            .search_keyword(
                "search",
                SearchDocumentQuery {
                    limit: Some(1),
                    ..SearchDocumentQuery::default()
                },
            )
            .await
            .expect("search first page");
        let second_page = repo
            .search_keyword(
                "search",
                SearchDocumentQuery {
                    limit: Some(1),
                    cursor: first_page.next_cursor.clone(),
                    ..SearchDocumentQuery::default()
                },
            )
            .await
            .expect("search second page");

        assert_eq!(first_page.results.len(), 1);
        assert!(first_page.has_more);
        assert!(first_page.next_cursor.is_some());
        assert_eq!(second_page.results.len(), 1);
        assert_ne!(first_page.results[0].id, second_page.results[0].id);

        cleanup_db(db_path);
    }

    #[tokio::test]
    async fn search_semantic_returns_indexed_embedding_results() {
        let (database, repo, db_path) = test_repo().await;
        seed_search_resources(&database).await;
        repo.reindex_all().await.expect("reindex all resources");
        SearchEmbeddingRepository::new(database.clone())
            .index_all_embeddings(DEFAULT_SEMANTIC_MODEL)
            .await
            .expect("index embeddings");

        let results = repo
            .search_semantic(
                "search testing",
                SearchDocumentQuery {
                    limit: Some(3),
                    ..SearchDocumentQuery::default()
                },
            )
            .await
            .expect("search semantic");

        assert!(!results.results.is_empty());
        assert!(results
            .results
            .iter()
            .all(|result| result.match_kind == "semantic"));

        cleanup_db(db_path);
    }

    #[tokio::test]
    async fn search_hybrid_merges_keyword_and_semantic_results() {
        let (database, repo, db_path) = test_repo().await;
        seed_search_resources(&database).await;
        repo.reindex_all().await.expect("reindex all resources");
        SearchEmbeddingRepository::new(database.clone())
            .index_all_embeddings(DEFAULT_SEMANTIC_MODEL)
            .await
            .expect("index embeddings");

        let results = repo
            .search_hybrid(
                "search",
                SearchDocumentQuery {
                    limit: Some(5),
                    ..SearchDocumentQuery::default()
                },
            )
            .await
            .expect("search hybrid");

        assert!(!results.results.is_empty());
        assert!(results
            .results
            .iter()
            .all(|result| result.match_kind == "hybrid"));

        cleanup_db(db_path);
    }

    #[tokio::test]
    async fn ranking_boosts_apply_domain_signals() {
        let (database, repo, db_path) = test_repo().await;
        seed_search_resources(&database).await;
        let conn = database.connect().expect("connect to test database");
        conn.execute(
            "UPDATE entries SET is_pinned = 1 WHERE id = ?1",
            libsql::params!["entry-1"],
        )
        .await
        .expect("pin entry");
        conn.execute(
            "INSERT INTO goal_instances (id, goal_id, period_start, period_end, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6)",
            libsql::params![
                "goal-instance-1",
                "goal-1",
                "2026-01-01T00:00:00Z",
                "active",
                "2026-01-01T00:00:00Z",
                "2026-01-01T00:00:00Z"
            ],
        )
        .await
        .expect("seed current goal instance");

        let mut task_result = test_search_result("task", "task-1", "Plan search testing", 0.1);
        repo.apply_ranking_boosts(&conn, "Plan search testing", &mut task_result)
            .await
            .expect("boost task");
        assert!(task_result.score > 0.39);

        let mut bookmark_result =
            test_search_result("bookmark", "bookmark-1", "Search reference", 0.1);
        repo.apply_ranking_boosts(&conn, "reflection", &mut bookmark_result)
            .await
            .expect("boost bookmark tag");
        assert!(bookmark_result.score >= 0.18);

        let mut entry_result = test_search_result("entry", "entry-1", "Morning clarity", 0.1);
        repo.apply_ranking_boosts(&conn, "clarity", &mut entry_result)
            .await
            .expect("boost pinned entry");
        assert!(entry_result.score >= 0.15);

        let mut goal_result = test_search_result("goal", "goal-1", "Improve recall", 0.1);
        repo.apply_ranking_boosts(&conn, "recall", &mut goal_result)
            .await
            .expect("boost current goal");
        assert!(goal_result.score >= 0.15);

        cleanup_db(db_path);
    }

    #[tokio::test]
    async fn find_related_returns_results_without_source_resource() {
        let (database, repo, db_path) = test_repo().await;
        seed_search_resources(&database).await;
        repo.reindex_all().await.expect("reindex all resources");

        let results = repo
            .find_related("task", "task-1", Some(5))
            .await
            .expect("find related resources");

        assert!(!results.is_empty());
        assert!(results
            .iter()
            .all(|result| result.resource_type != "task" || result.resource_id != "task-1"));

        cleanup_db(db_path);
    }

    fn test_search_result(
        resource_type: &str,
        resource_id: &str,
        title: &str,
        score: f64,
    ) -> SearchDocumentResult {
        SearchDocumentResult {
            id: format!("{}:{}:0", resource_type, resource_id),
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            title: title.to_string(),
            preview: title.to_string(),
            score,
            match_kind: "test".to_string(),
            highlights: Vec::new(),
            source_updated_at: "2026-05-18T00:00:00Z".to_string(),
            created_at: "2026-05-18T00:00:00Z".to_string(),
            updated_at: "2026-05-18T00:00:00Z".to_string(),
        }
    }

    #[tokio::test]
    async fn list_context_by_date_range_returns_clean_context() {
        let (database, repo, db_path) = test_repo().await;
        seed_search_resources(&database).await;
        repo.reindex_all().await.expect("reindex all resources");

        let results = repo
            .list_context_by_date_range("2026-05-12T00:00:00Z", "2026-05-15T23:59:59Z", Some(10))
            .await
            .expect("list context by date");

        assert_eq!(results.len(), 4);
        assert!(results.iter().all(|result| result.match_kind == "date"));
        assert!(results.iter().all(|result| !result.preview.is_empty()));

        cleanup_db(db_path);
    }

    #[tokio::test]
    async fn invalid_lexical_json_does_not_break_reindex_all() {
        let (database, repo, db_path) = test_repo().await;
        let conn = database.connect().expect("connect to test database");
        conn.execute(
            "INSERT INTO entries (id, document, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            libsql::params![
                "entry-invalid",
                "not valid lexical json",
                "2026-05-10T09:00:00Z",
                "2026-05-10T09:00:00Z"
            ],
        )
        .await
        .expect("seed invalid entry");

        let status = repo.reindex_all().await.expect("reindex invalid entry");

        assert_eq!(status.total_documents, 1);
        assert_eq!(status.entries, 1);

        cleanup_db(db_path);
    }
}
