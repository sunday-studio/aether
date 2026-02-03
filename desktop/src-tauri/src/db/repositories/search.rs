use crate::db::models::{Entry, Goal, SubTask, Tag, Task, Bookmark};
use crate::error::{AppError, Result};
use libsql::Database;
use std::sync::Arc;

/// Search result with resource type and relevance score
#[derive(Debug, Clone)]
pub enum SearchResult {
    Entry {
        entry: Entry,
        score: f64,
        highlights: Vec<String>,
    },
    Task {
        task: Task,
        score: f64,
        highlights: Vec<String>,
    },
    SubTask {
        subtask: SubTask,
        score: f64,
        highlights: Vec<String>,
    },
    Goal {
        goal: Goal,
        score: f64,
        highlights: Vec<String>,
    },
    Tag {
        tag: Tag,
        score: f64,
        highlights: Vec<String>,
    },
    Bookmark {
        bookmark: Bookmark,
        score: f64,
        highlights: Vec<String>,
    },
}

/// Resource type filter for search
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceType {
    Entry,
    Task,
    SubTask,
    Goal,
    Tag,
    Bookmark,
}

impl ResourceType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "entry" => Some(Self::Entry),
            "task" => Some(Self::Task),
            "subtask" => Some(Self::SubTask),
            "goal" => Some(Self::Goal),
            "tag" => Some(Self::Tag),
            "bookmark" => Some(Self::Bookmark),
            _ => None,
        }
    }
}

pub struct SearchRepository {
    database: Arc<Database>,
}

impl SearchRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Escape special characters in FTS5 query
    pub fn escape_fts_query(query: &str) -> String {
        query.replace('"', "\"\"")
    }

    /// Search across all resources using FTS5 fuzzy matching
    pub async fn search_fuzzy(
        &self,
        query: &str,
        types: Option<Vec<ResourceType>>,
        tag_ids: Option<Vec<String>>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<SearchResult>> {
        self.search_internal(query, types, tag_ids, limit, offset, "fuzzy").await
    }

    /// Search using hybrid mode (combines fuzzy and semantic search)
    pub async fn search_hybrid(
        &self,
        query: &str,
        types: Option<Vec<ResourceType>>,
        tag_ids: Option<Vec<String>>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<SearchResult>> {
        self.search_internal(query, types, tag_ids, limit, offset, "hybrid").await
    }

    /// Internal search method that handles different modes
    async fn search_internal(
        &self,
        query: &str,
        types: Option<Vec<ResourceType>>,
        tag_ids: Option<Vec<String>>,
        limit: Option<u32>,
        offset: Option<u32>,
        mode: &str,
    ) -> Result<Vec<SearchResult>> {
        if query.is_empty() {
            return Ok(Vec::new());
        }

        let limit = limit.unwrap_or(50).min(100);
        let _offset = offset.unwrap_or(0);
        let escaped_query = Self::escape_fts_query(query);

        let mut results = Vec::new();

        let search_types = if let Some(ref t) = types {
            t.clone()
        } else {
            vec![
                ResourceType::Entry,
                ResourceType::Task,
                ResourceType::SubTask,
                ResourceType::Goal,
                ResourceType::Tag,
                ResourceType::Bookmark,
            ]
        };

        if search_types.contains(&ResourceType::Entry) {
            let entry_results = self.search_entries(&escaped_query, &tag_ids, limit).await?;
            results.extend(entry_results);
        }

        if search_types.contains(&ResourceType::Task) {
            let task_results = self.search_tasks(&escaped_query, &tag_ids, limit).await?;
            results.extend(task_results);
        }

        if search_types.contains(&ResourceType::SubTask) {
            let subtask_results = self.search_subtasks(&escaped_query, limit).await?;
            results.extend(subtask_results);
        }

        if search_types.contains(&ResourceType::Goal) {
            let goal_results = self.search_goals(&escaped_query, &tag_ids, limit).await?;
            results.extend(goal_results);
        }

        if search_types.contains(&ResourceType::Tag) {
            let tag_results = self.search_tags(&escaped_query, limit).await?;
            results.extend(tag_results);
        }

        if search_types.contains(&ResourceType::Bookmark) {
            let bookmark_results = self.search_bookmarks(&escaped_query, &tag_ids, limit).await?;
            results.extend(bookmark_results);
        }

        results.sort_by(|a, b| {
            let score_a = match a {
                SearchResult::Entry { score, .. } => *score,
                SearchResult::Task { score, .. } => *score,
                SearchResult::SubTask { score, .. } => *score,
                SearchResult::Goal { score, .. } => *score,
                SearchResult::Tag { score, .. } => *score,
                SearchResult::Bookmark { score, .. } => *score,
            };
            let score_b = match b {
                SearchResult::Entry { score, .. } => *score,
                SearchResult::Task { score, .. } => *score,
                SearchResult::SubTask { score, .. } => *score,
                SearchResult::Goal { score, .. } => *score,
                SearchResult::Tag { score, .. } => *score,
                SearchResult::Bookmark { score, .. } => *score,
            };
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        if mode == "hybrid" {
            let semantic_results = self.search_semantic(query, &search_types, limit).await?;
            
            let mut merged: std::collections::HashMap<String, (SearchResult, f64, f64)> = std::collections::HashMap::new();
            
            for result in results {
                let (id, score) = match &result {
                    SearchResult::Entry { entry, score, .. } => (entry.id.clone(), *score),
                    SearchResult::Task { task, score, .. } => (task.id.clone(), *score),
                    SearchResult::SubTask { subtask, score, .. } => (subtask.id.clone(), *score),
                    SearchResult::Goal { goal, score, .. } => (goal.id.clone(), *score),
                    SearchResult::Tag { tag, score, .. } => (tag.id.clone(), *score),
                    SearchResult::Bookmark { bookmark, score, .. } => (bookmark.id.clone(), *score),
                };
                merged.insert(id, (result, score * 0.6, 0.0));
            }
            
            for result in semantic_results {
                let (id, score) = match &result {
                    SearchResult::Entry { entry, score, .. } => (entry.id.clone(), *score),
                    SearchResult::Task { task, score, .. } => (task.id.clone(), *score),
                    SearchResult::SubTask { subtask, score, .. } => (subtask.id.clone(), *score),
                    SearchResult::Goal { goal, score, .. } => (goal.id.clone(), *score),
                    SearchResult::Tag { tag, score, .. } => (tag.id.clone(), *score),
                    SearchResult::Bookmark { bookmark, score, .. } => (bookmark.id.clone(), *score),
                };
                
                match merged.get_mut(&id) {
                    Some((_, _, semantic_score)) => {
                        *semantic_score = score * 0.4;
                    }
                    None => {
                        merged.insert(id, (result, 0.0, score * 0.4));
                    }
                }
            }
            
            let mut final_results: Vec<SearchResult> = merged
                .into_iter()
                .map(|(_, (mut result, fuzzy_score, semantic_score))| {
                    let final_score = fuzzy_score + semantic_score;
                    match &mut result {
                        SearchResult::Entry { score, .. } => *score = final_score,
                        SearchResult::Task { score, .. } => *score = final_score,
                        SearchResult::SubTask { score, .. } => *score = final_score,
                        SearchResult::Goal { score, .. } => *score = final_score,
                        SearchResult::Tag { score, .. } => *score = final_score,
                        SearchResult::Bookmark { score, .. } => *score = final_score,
                    }
                    result
                })
                .collect();
            
            final_results.sort_by(|a, b| {
                let score_a = match a {
                    SearchResult::Entry { score, .. } => *score,
                    SearchResult::Task { score, .. } => *score,
                    SearchResult::SubTask { score, .. } => *score,
                    SearchResult::Goal { score, .. } => *score,
                    SearchResult::Tag { score, .. } => *score,
                    SearchResult::Bookmark { score, .. } => *score,
                };
                let score_b = match b {
                    SearchResult::Entry { score, .. } => *score,
                    SearchResult::Task { score, .. } => *score,
                    SearchResult::SubTask { score, .. } => *score,
                    SearchResult::Goal { score, .. } => *score,
                    SearchResult::Tag { score, .. } => *score,
                    SearchResult::Bookmark { score, .. } => *score,
                };
                score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
            });
            
            final_results.truncate(limit as usize);
            return Ok(final_results);
        }

        results.truncate(limit as usize);
        Ok(results)
    }

    async fn search_entries(
        &self,
        query: &str,
        _tag_ids: &Option<Vec<String>>,
        limit: u32,
    ) -> Result<Vec<SearchResult>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        let sql = "SELECT e.id, e.document, e.created_at, e.is_pinned, e.is_archived, e.is_deleted, e.updated_at, e.deleted_at,
                   bm25(entries_fts) as rank
                   FROM entries_fts
                   JOIN entries_fts_map m ON m.rowid = entries_fts.rowid
                   JOIN entries e ON e.id = m.entry_id
                   WHERE entries_fts MATCH ?1 AND e.deleted_at IS NULL
                   ORDER BY rank LIMIT ?2";

        let mut rows = conn
            .query(sql, libsql::params![query, limit as i64])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            let rank: f64 = row.get(8).unwrap_or(0.0);
            let entry = self.row_to_entry(row)?;
            let score = if rank > 0.0 { 1.0 / (1.0 + rank) } else { 1.0 };
            results.push(SearchResult::Entry {
                entry,
                score,
                highlights: Vec::new(),
            });
        }

        Ok(results)
    }

    async fn search_tasks(
        &self,
        query: &str,
        _tag_ids: &Option<Vec<String>>,
        limit: u32,
    ) -> Result<Vec<SearchResult>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        let sql = "SELECT t.id, t.title, t.description, t.is_completed, t.due_date, t.goal_instance_id, t.goal_id, 
                   t.created_at, t.updated_at, t.deleted_at,
                   bm25(tasks_fts) as rank
                   FROM tasks_fts
                   JOIN tasks_fts_map m ON m.rowid = tasks_fts.rowid
                   JOIN tasks t ON t.id = m.task_id
                   WHERE tasks_fts MATCH ?1 AND t.deleted_at IS NULL
                   ORDER BY rank LIMIT ?2";

        let mut rows = conn
            .query(sql, libsql::params![query, limit as i64])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            let rank: f64 = row.get(10).unwrap_or(0.0);
            let task = self.row_to_task(row)?;
            let score = if rank > 0.0 { 1.0 / (1.0 + rank) } else { 1.0 };
            results.push(SearchResult::Task {
                task,
                score,
                highlights: Vec::new(),
            });
        }

        Ok(results)
    }

    async fn search_subtasks(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<SearchResult>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        let sql = "SELECT s.id, s.title, s.is_completed, s.task_id, s.order_index, s.created_at, s.updated_at, s.deleted_at,
                   bm25(subtasks_fts) as rank
                   FROM subtasks_fts
                   JOIN subtasks_fts_map m ON m.rowid = subtasks_fts.rowid
                   JOIN subtasks s ON s.id = m.subtask_id
                   WHERE subtasks_fts MATCH ?1 AND s.deleted_at IS NULL
                   ORDER BY rank LIMIT ?2";

        let mut rows = conn
            .query(sql, libsql::params![query, limit as i64])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            let rank: f64 = row.get(8).unwrap_or(0.0);
            let subtask = self.row_to_subtask(row)?;
            let score = if rank > 0.0 { 1.0 / (1.0 + rank) } else { 1.0 };
            results.push(SearchResult::SubTask {
                subtask,
                score,
                highlights: Vec::new(),
            });
        }

        Ok(results)
    }

    async fn search_goals(
        &self,
        query: &str,
        _tag_ids: &Option<Vec<String>>,
        limit: u32,
    ) -> Result<Vec<SearchResult>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        let sql = "SELECT g.id, g.name, g.description, g.is_non_recurring, g.recurrence_type, g.recurrence_interval, 
                   g.recurrence_anchor, g.recurrence_meta, g.timezone, g.created_at, g.updated_at, g.deleted_at,
                   bm25(goals_fts) as rank
                   FROM goals_fts
                   JOIN goals_fts_map m ON m.rowid = goals_fts.rowid
                   JOIN goals g ON g.id = m.goal_id
                   WHERE goals_fts MATCH ?1 AND g.deleted_at IS NULL
                   ORDER BY rank LIMIT ?2";

        let mut rows = conn
            .query(sql, libsql::params![query, limit as i64])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            let rank: f64 = row.get(12).unwrap_or(0.0);
            let goal = self.row_to_goal(row)?;
            let score = if rank > 0.0 { 1.0 / (1.0 + rank) } else { 1.0 };
            results.push(SearchResult::Goal {
                goal,
                score,
                highlights: Vec::new(),
            });
        }

        Ok(results)
    }

    async fn search_tags(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<SearchResult>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        let sql = "SELECT t.id, t.name, t.created_at, t.updated_at, t.deleted_at,
                   bm25(tags_fts) as rank
                   FROM tags_fts
                   JOIN tags_fts_map m ON m.rowid = tags_fts.rowid
                   JOIN tags t ON t.id = m.tag_id
                   WHERE tags_fts MATCH ?1 AND t.deleted_at IS NULL
                   ORDER BY rank LIMIT ?2";

        let mut rows = conn
            .query(sql, libsql::params![query, limit as i64])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            let rank: f64 = row.get(5).unwrap_or(0.0);
            let tag = self.row_to_tag(row)?;
            let score = if rank > 0.0 { 1.0 / (1.0 + rank) } else { 1.0 };
            results.push(SearchResult::Tag {
                tag,
                score,
                highlights: Vec::new(),
            });
        }

        Ok(results)
    }

    async fn search_bookmarks(
        &self,
        query: &str,
        _tag_ids: &Option<Vec<String>>,
        limit: u32,
    ) -> Result<Vec<SearchResult>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        let sql = "SELECT b.id, b.url, b.title, b.description, b.image_url, b.favicon_url, b.site_name, b.author, b.published_at, b.content_type, b.metadata_json, b.is_archived, b.is_deleted, b.created_at, b.updated_at, b.deleted_at,
                   bm25(bookmarks_fts) as rank
                   FROM bookmarks_fts
                   JOIN bookmarks_fts_map m ON m.rowid = bookmarks_fts.rowid
                   JOIN bookmarks b ON b.id = m.bookmark_id
                   WHERE bookmarks_fts MATCH ?1 AND b.deleted_at IS NULL
                   ORDER BY rank LIMIT ?2";

        let mut rows = conn
            .query(sql, libsql::params![query, limit as i64])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            let rank: f64 = row.get(16).unwrap_or(0.0);
            let bookmark = self.row_to_bookmark(row)?;
            let score = if rank > 0.0 { 1.0 / (1.0 + rank) } else { 1.0 };
            results.push(SearchResult::Bookmark {
                bookmark,
                score,
                highlights: Vec::new(),
            });
        }

        Ok(results)
    }

    /// Search using semantic similarity (vector embeddings)
    async fn search_semantic(
        &self,
        query: &str,
        types: &[ResourceType],
        limit: u32,
    ) -> Result<Vec<SearchResult>> {
        let query_embedding = crate::utils::embeddings::generate_embedding(query).await?;
        let embedding_json = serde_json::to_string(&query_embedding)
            .map_err(|e| AppError::Internal(format!("Failed to serialize embedding: {}", e)))?;
        
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        let mut all_results = Vec::new();
        
        for resource_type in types {
            match resource_type {
                ResourceType::Entry => {
                    let sql = "SELECT e.id, e.document, e.created_at, e.is_pinned, e.is_archived, e.is_deleted, e.updated_at, e.deleted_at,
                               vector_distance_cos(e.embedding, vector32(?1)) as distance
                               FROM entries e
                               WHERE e.embedding IS NOT NULL AND e.deleted_at IS NULL
                               ORDER BY distance ASC
                               LIMIT ?2";
                    
                    let mut rows = conn
                        .query(sql, libsql::params![embedding_json.clone(), limit as i64])
                        .await
                        .map_err(|e| AppError::LibSQL(e))?;
                    
                    while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
                        let distance: f64 = row.get(8).unwrap_or(2.0);
                        let entry = self.row_to_entry(row)?;
                        let score = (1.0 - (distance / 2.0)).max(0.0);
                        all_results.push(SearchResult::Entry {
                            entry,
                            score,
                            highlights: Vec::new(),
                        });
                    }
                }
                ResourceType::Task => {
                    let sql = "SELECT t.id, t.title, t.description, t.is_completed, t.due_date, t.goal_instance_id, t.goal_id, 
                               t.created_at, t.updated_at, t.deleted_at,
                               vector_distance_cos(t.embedding, vector32(?1)) as distance
                               FROM tasks t
                               WHERE t.embedding IS NOT NULL AND t.deleted_at IS NULL
                               ORDER BY distance ASC
                               LIMIT ?2";
                    
                    let mut rows = conn
                        .query(sql, libsql::params![embedding_json.clone(), limit as i64])
                        .await
                        .map_err(|e| AppError::LibSQL(e))?;
                    
                    while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
                        let distance: f64 = row.get(10).unwrap_or(2.0);
                        let task = self.row_to_task(row)?;
                        let score = (1.0 - (distance / 2.0)).max(0.0);
                        all_results.push(SearchResult::Task {
                            task,
                            score,
                            highlights: Vec::new(),
                        });
                    }
                }
                ResourceType::SubTask => {
                    let sql = "SELECT s.id, s.title, s.is_completed, s.task_id, s.order_index, s.created_at, s.updated_at, s.deleted_at,
                               vector_distance_cos(s.embedding, vector32(?1)) as distance
                               FROM subtasks s
                               WHERE s.embedding IS NOT NULL AND s.deleted_at IS NULL
                               ORDER BY distance ASC
                               LIMIT ?2";
                    
                    let mut rows = conn
                        .query(sql, libsql::params![embedding_json.clone(), limit as i64])
                        .await
                        .map_err(|e| AppError::LibSQL(e))?;
                    
                    while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
                        let distance: f64 = row.get(8).unwrap_or(2.0);
                        let subtask = self.row_to_subtask(row)?;
                        let score = (1.0 - (distance / 2.0)).max(0.0);
                        all_results.push(SearchResult::SubTask {
                            subtask,
                            score,
                            highlights: Vec::new(),
                        });
                    }
                }
                ResourceType::Goal => {
                    let sql = "SELECT g.id, g.name, g.description, g.is_non_recurring, g.recurrence_type, g.recurrence_interval, 
                               g.recurrence_anchor, g.recurrence_meta, g.timezone, g.created_at, g.updated_at, g.deleted_at,
                               vector_distance_cos(g.embedding, vector32(?1)) as distance
                               FROM goals g
                               WHERE g.embedding IS NOT NULL AND g.deleted_at IS NULL
                               ORDER BY distance ASC
                               LIMIT ?2";
                    
                    let mut rows = conn
                        .query(sql, libsql::params![embedding_json.clone(), limit as i64])
                        .await
                        .map_err(|e| AppError::LibSQL(e))?;
                    
                    while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
                        let distance: f64 = row.get(12).unwrap_or(2.0);
                        let goal = self.row_to_goal(row)?;
                        let score = (1.0 - (distance / 2.0)).max(0.0);
                        all_results.push(SearchResult::Goal {
                            goal,
                            score,
                            highlights: Vec::new(),
                        });
                    }
                }
                ResourceType::Tag => {
                    let sql = "SELECT t.id, t.name, t.created_at, t.updated_at, t.deleted_at,
                               vector_distance_cos(t.embedding, vector32(?1)) as distance
                               FROM tags t
                               WHERE t.embedding IS NOT NULL AND t.deleted_at IS NULL
                               ORDER BY distance ASC
                               LIMIT ?2";
                    
                    let mut rows = conn
                        .query(sql, libsql::params![embedding_json.clone(), limit as i64])
                        .await
                        .map_err(|e| AppError::LibSQL(e))?;
                    
                    while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
                        let distance: f64 = row.get(5).unwrap_or(2.0);
                        let tag = self.row_to_tag(row)?;
                        let score = (1.0 - (distance / 2.0)).max(0.0);
                        all_results.push(SearchResult::Tag {
                            tag,
                            score,
                            highlights: Vec::new(),
                        });
                    }
                }
                ResourceType::Bookmark => {
                    let sql = "SELECT b.id, b.url, b.title, b.description, b.image_url, b.favicon_url, b.site_name, b.author, b.published_at, b.content_type, b.metadata_json, b.is_archived, b.is_deleted, b.created_at, b.updated_at, b.deleted_at,
                               vector_distance_cos(b.embedding, vector32(?1)) as distance
                               FROM bookmarks b
                               WHERE b.embedding IS NOT NULL AND b.deleted_at IS NULL
                               ORDER BY distance ASC
                               LIMIT ?2";
                    
                    let mut rows = conn
                        .query(sql, libsql::params![embedding_json.clone(), limit as i64])
                        .await
                        .map_err(|e| AppError::LibSQL(e))?;
                    
                    while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
                        let distance: f64 = row.get(16).unwrap_or(2.0);
                        let bookmark = self.row_to_bookmark(row)?;
                        let score = (1.0 - (distance / 2.0)).max(0.0);
                        all_results.push(SearchResult::Bookmark {
                            bookmark,
                            score,
                            highlights: Vec::new(),
                        });
                    }
                }
            }
        }
        
        Ok(all_results)
    }

    /// Search for similar resources using vector embeddings
    pub async fn search_similar(
        &self,
        resource_type: &str,
        resource_id: &str,
        limit: u32,
    ) -> Result<Vec<SearchResult>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        let limit = limit.min(50);

        match resource_type {
            "entry" => {
                let sql = "SELECT e.id, e.document, e.created_at, e.is_pinned, e.is_archived, e.is_deleted, e.updated_at, e.deleted_at,
                           vector_distance_cos(e.embedding, (SELECT embedding FROM entries WHERE id = ?3)) as distance
                           FROM vector_top_k('entries_embedding_idx', (SELECT embedding FROM entries WHERE id = ?3), ?2) v
                           JOIN entries e ON e.rowid = v.id
                           WHERE e.id != ?3 AND e.deleted_at IS NULL
                           ORDER BY distance ASC";

                let mut rows = conn
                    .query(sql, libsql::params![limit as i64, resource_id, resource_id])
                    .await
                    .map_err(|e| AppError::LibSQL(e))?;

                let mut results = Vec::new();
                while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
                    let distance: f64 = row.get(8).unwrap_or(2.0);
                    let entry = self.row_to_entry(row)?;
                    let score = (1.0 - (distance / 2.0)).max(0.0);
                    results.push(SearchResult::Entry {
                        entry,
                        score,
                        highlights: Vec::new(),
                    });
                }
                Ok(results)
            }
            "task" | "subtask" | "goal" | "tag" => {
                // Similar implementation for other types
                // For brevity, returning empty for now - can be extended
                Ok(Vec::new())
            }
            _ => Err(AppError::BadRequest(format!("Invalid resource type: {}", resource_type))),
        }
    }

    fn row_to_entry(&self, row: libsql::Row) -> Result<Entry> {
        Ok(Entry {
            id: row.get(0)?,
            document: row.get(1)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String>(2)?)
                .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))?
                .with_timezone(&chrono::Utc),
            is_pinned: row.get::<i64>(3)? != 0,
            is_archived: row.get::<i64>(4)? != 0,
            is_deleted: row.get::<i64>(5)? != 0,
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String>(6)?)
                .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))?
                .with_timezone(&chrono::Utc),
            deleted_at: row.get::<Option<String>>(7)?
                .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))
                    .map(|dt| dt.with_timezone(&chrono::Utc)))
                .transpose()?,
            _sync_id: row.get::<String>(8).ok(),
            _updated_at: row.get::<i64>(9).ok(),
            _deleted: row.get::<i64>(10).ok().map(|v| v != 0).unwrap_or(false),
            _extra: row.get::<Option<String>>(11).ok().flatten().and_then(|s| serde_json::from_str(&s).ok()),
        })
    }

    fn row_to_task(&self, row: libsql::Row) -> Result<Task> {
        Ok(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            is_completed: row.get::<i64>(3)? != 0,
            due_date: row.get::<Option<String>>(4)?
                .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))
                    .map(|dt| dt.with_timezone(&chrono::Utc)))
                .transpose()?,
            goal_instance_id: row.get(5)?,
            goal_id: row.get(6)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String>(7)?)
                .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))?
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String>(8)?)
                .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))?
                .with_timezone(&chrono::Utc),
            deleted_at: row.get::<Option<String>>(9)?
                .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))
                    .map(|dt| dt.with_timezone(&chrono::Utc)))
                .transpose()?,
            subtasks: None,
            _sync_id: None,
            _updated_at: None,
            _deleted: false,
            _extra: None,
        })
    }

    fn row_to_subtask(&self, row: libsql::Row) -> Result<SubTask> {
        Ok(SubTask {
            id: row.get(0)?,
            title: row.get(1)?,
            is_completed: row.get::<i64>(2)? != 0,
            task_id: row.get(3)?,
            order_index: row.get(4)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String>(5)?)
                .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))?
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String>(6)?)
                .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))?
                .with_timezone(&chrono::Utc),
            deleted_at: row.get::<Option<String>>(7)?
                .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))
                    .map(|dt| dt.with_timezone(&chrono::Utc)))
                .transpose()?,
            _sync_id: None,
            _updated_at: None,
            _deleted: false,
            _extra: None,
        })
    }

    fn row_to_goal(&self, row: libsql::Row) -> Result<Goal> {
        Ok(Goal {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            is_non_recurring: row.get::<i64>(3)? != 0,
            recurrence_type: row.get(4)?,
            recurrence_interval: row.get(5)?,
            recurrence_anchor: row.get::<Option<String>>(6)?
                .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))
                    .map(|dt| dt.with_timezone(&chrono::Utc)))
                .transpose()?,
            recurrence_meta: row.get::<Option<String>>(7)?
                .map(|s| serde_json::from_str(&s).unwrap_or(serde_json::Value::Null)),
            timezone: row.get(8)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String>(9)?)
                .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))?
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String>(10)?)
                .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))?
                .with_timezone(&chrono::Utc),
            deleted_at: row.get::<Option<String>>(11)?
                .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))
                    .map(|dt| dt.with_timezone(&chrono::Utc)))
                .transpose()?,
            _sync_id: None,
            _updated_at: None,
            _deleted: false,
            _extra: None,
        })
    }

    fn row_to_tag(&self, row: libsql::Row) -> Result<Tag> {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String>(2)?)
                .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))?
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String>(3)?)
                .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))?
                .with_timezone(&chrono::Utc),
            deleted_at: row.get::<Option<String>>(4)?
                .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))
                    .map(|dt| dt.with_timezone(&chrono::Utc)))
                .transpose()?,
            _sync_id: None,
            _updated_at: None,
            _deleted: false,
            _extra: None,
        })
    }

    fn row_to_bookmark(&self, row: libsql::Row) -> Result<Bookmark> {
        let id: String = row.get(0)?;
        let url: String = row.get(1)?;
        let title: Option<String> = row.get(2)?;
        let description: Option<String> = row.get(3)?;
        let image_url: Option<String> = row.get(4)?;
        let favicon_url: Option<String> = row.get(5)?;
        let site_name: Option<String> = row.get(6)?;
        let author: Option<String> = row.get(7)?;
        let published_at_str: Option<String> = row.get(8)?;
        let content_type: Option<String> = row.get(9)?;
        let metadata_json_str: Option<String> = row.get(10)?;
        let is_archived: i64 = row.get(11)?;
        let is_deleted: i64 = row.get(12)?;
        let created_at_str: String = row.get(13)?;
        let updated_at_str: String = row.get(14)?;
        let deleted_at_str: Option<String> = row.get(15)?;

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))?
            .with_timezone(&chrono::Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid datetime: {}", e)))?
            .with_timezone(&chrono::Utc);
        let deleted_at = deleted_at_str
            .map(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .flatten()
            .map(|dt| dt.with_timezone(&chrono::Utc));
        let published_at = published_at_str
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));
        let metadata_json = metadata_json_str
            .and_then(|s| serde_json::from_str(&s).ok());

        // Search queries (FTS, semantic) do not select sync columns; use defaults
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
            _sync_id: None,
            _updated_at: None,
            _deleted: false,
            _extra: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_type_from_str() {
        assert_eq!(ResourceType::from_str("entry"), Some(ResourceType::Entry));
        assert_eq!(ResourceType::from_str("task"), Some(ResourceType::Task));
        assert_eq!(ResourceType::from_str("subtask"), Some(ResourceType::SubTask));
        assert_eq!(ResourceType::from_str("goal"), Some(ResourceType::Goal));
        assert_eq!(ResourceType::from_str("tag"), Some(ResourceType::Tag));
        assert_eq!(ResourceType::from_str("bookmark"), Some(ResourceType::Bookmark));
        assert_eq!(ResourceType::from_str("invalid"), None);
        assert_eq!(ResourceType::from_str("ENTRY"), Some(ResourceType::Entry));
    }

    #[test]
    fn test_escape_fts_query() {
        assert_eq!(SearchRepository::escape_fts_query("test"), "test");
        assert_eq!(SearchRepository::escape_fts_query("test\"quote"), "test\"\"quote");
    }
}
