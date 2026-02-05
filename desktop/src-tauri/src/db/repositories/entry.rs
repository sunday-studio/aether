use crate::db::models::{Entry, Tag};
use crate::error::{AppError, Result};
use crate::utils::generate_id;
use chrono::Utc;
use libsql::Database;
use std::collections::HashMap;
use std::sync::Arc;

pub struct EntryRepository {
    database: Arc<Database>,
}

impl EntryRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Get all entries (non-deleted)
    /// If limit and cursor are both None, returns all entries (bypass pagination)
    /// Otherwise returns paginated results with cursor-based pagination
    pub async fn find_all(
        &self,
        limit: Option<u32>,
        cursor: Option<String>,
    ) -> Result<(Vec<Entry>, Option<String>, bool)> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Bypass mode: return all results
        if limit.is_none() && cursor.is_none() {
            let mut rows = conn
                .query(
                    "SELECT id, document, created_at, is_pinned, is_archived, is_deleted, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                     FROM entries 
                     WHERE is_deleted = 0 
                     ORDER BY id ASC",
                    libsql::params![],
                )
                .await
                .map_err(|e| AppError::LibSQL(e))?;

            let mut entries = Vec::new();
            while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
                entries.push(self.row_to_entry(row)?);
            }
            let ids: Vec<String> = entries.iter().map(|e| e.id.clone()).collect();
            let tag_map = self.get_tags_for_entries(&ids).await?;
            for entry in &mut entries {
                entry.tags = Some(tag_map.get(&entry.id).cloned().unwrap_or_default());
            }
            return Ok((entries, None, false));
        }

        // Pagination mode
        let limit_val = limit.unwrap_or(50).min(1000);
        let fetch_limit = limit_val + 1; // Fetch one extra to determine has_more
        
        let mut rows = if let Some(cursor_val) = cursor {
            // Decode cursor to get last ID
            use crate::handlers::common::cursor;
            let last_id = cursor::decode(&cursor_val)?;
            
            conn.query(
                "SELECT id, document, created_at, is_pinned, is_archived, is_deleted, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM entries 
                 WHERE is_deleted = 0 AND id > ?1
                 ORDER BY id ASC
                 LIMIT ?2",
                libsql::params![last_id, fetch_limit as i64],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?
        } else {
            conn.query(
                "SELECT id, document, created_at, is_pinned, is_archived, is_deleted, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM entries 
                 WHERE is_deleted = 0 
                 ORDER BY id ASC
                 LIMIT ?1",
                libsql::params![fetch_limit as i64],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?
        };

        let mut entries = Vec::new();
        let mut has_more = false;
        
        let mut count = 0;
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            if count < limit_val {
                entries.push(self.row_to_entry(row)?);
                count += 1;
            } else {
                // We have one extra row, so there are more results
                has_more = true;
                break;
            }
        }

        // Set next_cursor from last entry if we have entries
        let next_cursor = if has_more && !entries.is_empty() {
            use crate::handlers::common::cursor;
            Some(cursor::encode(&entries.last().unwrap().id))
        } else {
            None
        };

        let ids: Vec<String> = entries.iter().map(|e| e.id.clone()).collect();
        let tag_map = self.get_tags_for_entries(&ids).await?;
        for entry in &mut entries {
            entry.tags = Some(tag_map.get(&entry.id).cloned().unwrap_or_default());
        }

        Ok((entries, next_cursor, has_more))
    }

    /// Get entry by ID
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Entry>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT id, document, created_at, is_pinned, is_archived, is_deleted, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM entries 
                 WHERE id = ?1 AND is_deleted = 0",
                libsql::params![id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        if let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            let mut entry = self.row_to_entry(row)?;
            entry.tags = Some(self.get_tags_for_entry(&entry.id).await?);
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    /// Create a new entry
    pub async fn create(
        &self,
        document: String,
        created_at: chrono::DateTime<Utc>,
        is_pinned: bool,
        is_archived: bool,
        is_deleted: bool,
    ) -> Result<Entry> {
        // #region agent log
        let log_path = "/Users/casprine/Desktop/vendor/sunday-studio/aether/.cursor/debug.log";
        let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "{{\"id\":\"log_entry_create_start\",\"timestamp\":{},\"location\":\"entry.rs:70\",\"message\":\"Entry create started\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"A\"}}", 
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
        });
        // #endregion
        
        let conn = self.database.connect().map_err(|e| {
            // #region agent log
            let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                use std::io::Write;
                writeln!(f, "{{\"id\":\"log_connect_error\",\"timestamp\":{},\"location\":\"entry.rs:75\",\"message\":\"Database connect failed\",\"data\":{{\"error\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"A\"}}", 
                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                    e)
            });
            // #endregion
            AppError::LibSQL(e)
        })?;
        
        // #region agent log
        let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "{{\"id\":\"log_connect_success\",\"timestamp\":{},\"location\":\"entry.rs:78\",\"message\":\"Database connect succeeded\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"A\"}}", 
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
        });
        // #endregion
        
        let id = generate_id("entry");
        let now = Utc::now();
        let created_at_str = created_at.to_rfc3339();
        let updated_at_str = now.to_rfc3339();

        // #region agent log
        let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "{{\"id\":\"log_execute_before\",\"timestamp\":{},\"location\":\"entry.rs:90\",\"message\":\"About to execute INSERT\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"A\"}}", 
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
        });
        // #endregion

        let now_ms = now.timestamp_millis();
        
        // #region agent log - Check triggers exist (Hypothesis A)
        let log_path = "/Users/casprine/Desktop/vendor/sunday-studio/aether/.cursor/debug.log";
        let trigger_check = conn.query(
            "SELECT name FROM sqlite_master WHERE type='trigger' AND name LIKE 'entries_sync%'",
            libsql::params![]
        ).await;
        match trigger_check {
            Ok(mut rows) => {
                let mut trigger_names = Vec::new();
                while let Ok(Some(row)) = rows.next().await {
                    if let Ok(name) = row.get::<String>(0) {
                        trigger_names.push(name);
                    }
                }
                let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    use std::io::Write;
                    writeln!(f, "{{\"id\":\"log_triggers_check\",\"timestamp\":{},\"location\":\"entry.rs:112\",\"message\":\"Triggers found\",\"data\":{{\"triggers\":{:?}}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"A\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                        trigger_names)
                });
            }
            Err(e) => {
                let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    use std::io::Write;
                    writeln!(f, "{{\"id\":\"log_triggers_check_error\",\"timestamp\":{},\"location\":\"entry.rs:112\",\"message\":\"Failed to check triggers\",\"data\":{{\"error\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"A\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                        e)
                });
            }
        }
        // #endregion
        
        // #region agent log - Check _suppress_triggers value (Hypothesis C)
        let suppress_check = conn.query(
            "SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers'",
            libsql::params![]
        ).await;
        match suppress_check {
            Ok(mut rows) => {
                let suppress_value: String = if let Ok(Some(row)) = rows.next().await {
                    row.get::<String>(0).unwrap_or_else(|_| "0".to_string())
                } else {
                    "0".to_string()
                };
                let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    use std::io::Write;
                    writeln!(f, "{{\"id\":\"log_suppress_value\",\"timestamp\":{},\"location\":\"entry.rs:112\",\"message\":\"_suppress_triggers value\",\"data\":{{\"value\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"C\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                        suppress_value)
                });
            }
            Err(e) => {
                let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    use std::io::Write;
                    writeln!(f, "{{\"id\":\"log_suppress_check_error\",\"timestamp\":{},\"location\":\"entry.rs:112\",\"message\":\"Failed to check _suppress_triggers\",\"data\":{{\"error\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"C\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                        e)
                });
            }
        }
        // #endregion
        
        // #region agent log - Check outbox count before INSERT (Hypothesis D)
        let outbox_before = conn.query("SELECT COUNT(*) FROM _sync_outbox", libsql::params![]).await;
        match outbox_before {
            Ok(mut rows) => {
                let count: i64 = if let Ok(Some(row)) = rows.next().await {
                    row.get(0).unwrap_or(0)
                } else {
                    0
                };
                let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    use std::io::Write;
                    writeln!(f, "{{\"id\":\"log_outbox_before\",\"timestamp\":{},\"location\":\"entry.rs:112\",\"message\":\"Outbox count before INSERT\",\"data\":{{\"count\":{},\"entry_id\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"D\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                        count, id)
                });
            }
            Err(e) => {
                let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    use std::io::Write;
                    writeln!(f, "{{\"id\":\"log_outbox_before_error\",\"timestamp\":{},\"location\":\"entry.rs:112\",\"message\":\"Failed to check outbox before\",\"data\":{{\"error\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"D\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                        e)
                });
            }
        }
        // #endregion
        
        let execute_result = conn.execute(
            "INSERT INTO entries (id, document, created_at, is_pinned, is_archived, is_deleted, updated_at, _sync_id, _updated_at, _deleted, _extra) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            libsql::params![
                id.clone(),
                document.clone(),
                created_at_str,
                if is_pinned { 1 } else { 0 },
                if is_archived { 1 } else { 0 },
                if is_deleted { 1 } else { 0 },
                updated_at_str,
                id.clone(),
                now_ms,
                if is_deleted { 1 } else { 0 },
                "{}"
            ],
        )
        .await;
        
        // #region agent log - Check outbox count after INSERT (Hypothesis D)
        let outbox_after = conn.query("SELECT COUNT(*) FROM _sync_outbox", libsql::params![]).await;
        match outbox_after {
            Ok(mut rows) => {
                let count: i64 = if let Ok(Some(row)) = rows.next().await {
                    row.get(0).unwrap_or(0)
                } else {
                    0
                };
                let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    use std::io::Write;
                    writeln!(f, "{{\"id\":\"log_outbox_after\",\"timestamp\":{},\"location\":\"entry.rs:112\",\"message\":\"Outbox count after INSERT\",\"data\":{{\"count\":{},\"entry_id\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"D\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                        count, id)
                });
            }
            Err(e) => {
                let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    use std::io::Write;
                    writeln!(f, "{{\"id\":\"log_outbox_after_error\",\"timestamp\":{},\"location\":\"entry.rs:112\",\"message\":\"Failed to check outbox after\",\"data\":{{\"error\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"D\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                        e)
                });
            }
        }
        // #endregion
        
        // #region agent log - Check if entry was added to outbox (Hypothesis E)
        let outbox_entry = conn.query(
            "SELECT entity, entity_id, op FROM _sync_outbox WHERE entity='entries' AND entity_id=?1",
            libsql::params![id.clone()]
        ).await;
        match outbox_entry {
            Ok(mut rows) => {
                if let Ok(Some(row)) = rows.next().await {
                    let entity: String = row.get(0).unwrap_or_default();
                    let entity_id: String = row.get(1).unwrap_or_default();
                    let op: String = row.get(2).unwrap_or_default();
                    let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                        use std::io::Write;
                        writeln!(f, "{{\"id\":\"log_outbox_entry_found\",\"timestamp\":{},\"location\":\"entry.rs:112\",\"message\":\"Entry found in outbox\",\"data\":{{\"entity\":\"{}\",\"entity_id\":\"{}\",\"op\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"E\"}}", 
                            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                            entity, entity_id, op)
                    });
                } else {
                    let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                        use std::io::Write;
                        writeln!(f, "{{\"id\":\"log_outbox_entry_not_found\",\"timestamp\":{},\"location\":\"entry.rs:112\",\"message\":\"Entry NOT found in outbox\",\"data\":{{\"entry_id\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"E\"}}", 
                            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                            id)
                    });
                }
            }
            Err(e) => {
                let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    use std::io::Write;
                    writeln!(f, "{{\"id\":\"log_outbox_entry_check_error\",\"timestamp\":{},\"location\":\"entry.rs:112\",\"message\":\"Failed to check outbox entry\",\"data\":{{\"error\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"E\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                        e)
                });
            }
        }
        // #endregion
        
        match &execute_result {
            Ok(_) => {
                // #region agent log
                let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    use std::io::Write;
                    writeln!(f, "{{\"id\":\"log_execute_success\",\"timestamp\":{},\"location\":\"entry.rs:105\",\"message\":\"INSERT executed successfully\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"A\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
                });
                // #endregion
            }
            Err(e) => {
                // #region agent log
                let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    use std::io::Write;
                    writeln!(f, "{{\"id\":\"log_execute_error\",\"timestamp\":{},\"location\":\"entry.rs:110\",\"message\":\"INSERT failed\",\"data\":{{\"error\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"A\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                        e)
                });
                // #endregion
            }
        }
        
        execute_result.map_err(|e| AppError::LibSQL(e))?;

        Ok(Entry {
            id: id.clone(),
            document,
            created_at,
            is_pinned,
            is_archived,
            is_deleted,
            updated_at: now,
            deleted_at: None,
            tags: None,
            _sync_id: Some(id),
            _updated_at: Some(now_ms),
            _deleted: is_deleted,
            _extra: None,
        })
    }

    /// Bulk create entries
    pub async fn bulk_create(&self, entries_data: Vec<(String, chrono::DateTime<Utc>, bool, bool, bool)>) -> Result<Vec<Entry>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        conn.execute("BEGIN TRANSACTION", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut entries = Vec::new();
        let now = Utc::now();

        for (document, created_at, is_pinned, is_archived, is_deleted) in entries_data {
            let id = generate_id("entry");
            let created_at_str = created_at.to_rfc3339();
            let updated_at_str = now.to_rfc3339();

            let now_ms = now.timestamp_millis();
            conn.execute(
                "INSERT INTO entries (id, document, created_at, is_pinned, is_archived, is_deleted, updated_at, _sync_id, _updated_at, _deleted, _extra) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                libsql::params![
                    id.clone(),
                    document.clone(),
                    created_at_str,
                    if is_pinned { 1 } else { 0 },
                    if is_archived { 1 } else { 0 },
                    if is_deleted { 1 } else { 0 },
                    updated_at_str,
                    id.clone(),
                    now_ms,
                    if is_deleted { 1 } else { 0 },
                    "{}"
                ],
            )
            .await
            .map_err(|e| {
                let _ = conn.execute("ROLLBACK", libsql::params![]);
                AppError::LibSQL(e)
            })?;

            entries.push(Entry {
                id: id.clone(),
                document,
                created_at,
                is_pinned,
                is_archived,
                is_deleted,
                updated_at: now,
                deleted_at: None,
                tags: None,
                _sync_id: Some(id),
                _updated_at: Some(now_ms),
                _deleted: is_deleted,
                _extra: None,
            });
        }

        conn.execute("COMMIT", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        Ok(entries)
    }

    /// Update an entry
    pub async fn update(
        &self,
        id: &str,
        document: String,
        is_pinned: bool,
        is_archived: bool,
        is_deleted: bool,
        client_updated_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<Entry> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Get current entry
        let current = self.find_by_id(id).await?;
        let mut entry = current.ok_or_else(|| AppError::NotFound(format!("Entry {} not found", id)))?;

        // Last-Write-Wins conflict detection
        if let Some(client_time) = client_updated_at {
            if client_time < entry.updated_at {
                return Err(AppError::BadRequest(format!(
                    "Conflict: Record was modified by another device. Current updated_at: {}",
                    entry.updated_at.to_rfc3339()
                )));
            }
        }

        // Update fields
        entry.document = document;
        entry.is_pinned = is_pinned;
        entry.is_archived = is_archived;
        entry.is_deleted = is_deleted;
        entry.updated_at = Utc::now();
        let now_ms = entry.updated_at.timestamp_millis();
        entry._updated_at = Some(now_ms);
        entry._deleted = is_deleted;

        let updated_at_str = entry.updated_at.to_rfc3339();

        conn.execute(
            "UPDATE entries 
             SET document = ?1, is_pinned = ?2, is_archived = ?3, is_deleted = ?4, updated_at = ?5, _updated_at = ?6, _deleted = ?7 
             WHERE id = ?8",
            libsql::params![
                entry.document.clone(),
                if entry.is_pinned { 1 } else { 0 },
                if entry.is_archived { 1 } else { 0 },
                if entry.is_deleted { 1 } else { 0 },
                updated_at_str,
                now_ms,
                if entry.is_deleted { 1 } else { 0 },
                id
            ],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(entry)
    }

    /// Delete an entry (soft delete)
    pub async fn delete(&self, id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Check if entry exists
        let entry = self.find_by_id(id).await?;
        if entry.is_none() {
            return Err(AppError::NotFound(format!("Entry {} not found", id)));
        }

        let now = Utc::now();
        let updated_at_str = now.to_rfc3339();
        let now_ms = now.timestamp_millis();

        conn.execute(
            "UPDATE entries SET is_deleted = 1, updated_at = ?1, _updated_at = ?2, _deleted = 1 WHERE id = ?3",
            libsql::params![updated_at_str, now_ms, id],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Add tags to an entry
    pub async fn add_tags(&self, entry_id: &str, tag_ids: Vec<String>) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Verify entry exists
        let entry = self.find_by_id(entry_id).await?;
        if entry.is_none() {
            return Err(AppError::NotFound(format!("Entry {} not found", entry_id)));
        }

        if tag_ids.is_empty() {
            return Ok(());
        }

        // Verify tags exist - use single query with IN clause to avoid N+1
        // Use safe string interpolation with proper escaping
        let escaped_ids: Vec<String> = tag_ids
            .iter()
            .map(|id| format!("'{}'", id.replace("'", "''")))
            .collect();
        let query = format!("SELECT id FROM tags WHERE id IN ({})", escaped_ids.join(", "));
        
        let mut rows = conn
            .query(&query, libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;
        
        let mut found_tag_ids = std::collections::HashSet::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            if let Ok(tag_id) = row.get::<String>(0) {
                found_tag_ids.insert(tag_id);
            }
        }
        
        // Check if all tags were found
        for tag_id in &tag_ids {
            if !found_tag_ids.contains(tag_id) {
                return Err(AppError::NotFound(format!("Tag {} not found", tag_id)));
            }
        }

        // Insert tag associations (skip if already exists)
        conn.execute("BEGIN TRANSACTION", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let now_ms = Utc::now().timestamp_millis();
        for tag_id in &tag_ids {
            let sync_id = format!("{}|{}", entry_id, tag_id);
            conn.execute(
                "INSERT OR IGNORE INTO entry_tags (entry_id, tag_id, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, 0, '{}')",
                libsql::params![entry_id, tag_id.as_str(), sync_id, now_ms],
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

    /// Remove tags from an entry
    pub async fn remove_tags(&self, entry_id: &str, tag_ids: Vec<String>) -> Result<()> {
        if tag_ids.is_empty() {
            return Ok(());
        }
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        // Verify entry exists
        let entry = self.find_by_id(entry_id).await?;
        if entry.is_none() {
            return Err(AppError::NotFound(format!("Entry {} not found", entry_id)));
        }

        for tag_id in &tag_ids {
            // Verify tag exists
            let mut rows = conn
                .query("SELECT id FROM tags WHERE id = ?1", libsql::params![tag_id.as_str()])
                .await
                .map_err(|e| AppError::LibSQL(e))?;

            if rows.next().await.map_err(|e| AppError::LibSQL(e))?.is_none() {
                return Err(AppError::NotFound(format!("Tag {} not found", tag_id)));
            }

            conn.execute(
                "DELETE FROM entry_tags WHERE entry_id = ?1 AND tag_id = ?2",
                libsql::params![entry_id, tag_id.as_str()],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;
        }

        Ok(())
    }

    /// Helper to convert database row to Entry
    fn row_to_entry(&self, row: libsql::Row) -> Result<Entry> {
        let id: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
        let document: String = row.get(1).map_err(|e| AppError::LibSQL(e))?;
        let created_at_str: String = row.get(2).map_err(|e| AppError::LibSQL(e))?;
        let is_pinned: i64 = row.get(3).map_err(|e| AppError::LibSQL(e))?;
        let is_archived: i64 = row.get(4).map_err(|e| AppError::LibSQL(e))?;
        let is_deleted: i64 = row.get(5).map_err(|e| AppError::LibSQL(e))?;
        let updated_at_str: String = row.get(6).map_err(|e| AppError::LibSQL(e))?;
        let deleted_at_str: Option<String> = row.get(7).map_err(|e| AppError::LibSQL(e))?;
        let _sync_id: Option<String> = row.get(8).ok();
        let _updated_at: Option<i64> = row.get(9).ok();
        let _deleted: i64 = row.get(10).unwrap_or(0);
        let _extra: Option<serde_json::Value> = row.get::<Option<String>>(11).ok().flatten().and_then(|s| serde_json::from_str(&s).ok());

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

        Ok(Entry {
            id,
            document,
            created_at,
            is_pinned: is_pinned != 0,
            is_archived: is_archived != 0,
            is_deleted: is_deleted != 0,
            updated_at,
            deleted_at,
            tags: None,
            _sync_id,
            _updated_at,
            _deleted: _deleted != 0,
            _extra,
        })
    }

    /// Load tags for a single entry (for get_entry_by_id).
    pub async fn get_tags_for_entry(&self, entry_id: &str) -> Result<Vec<Tag>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        let mut rows = conn
            .query(
                "SELECT t.id, t.name, t.created_at, t.updated_at, t.deleted_at, t._sync_id, t._updated_at, t._deleted, t._extra
                 FROM tags t
                 INNER JOIN entry_tags et ON t.id = et.tag_id
                 WHERE et.entry_id = ?1 AND t.deleted_at IS NULL
                 ORDER BY t.name ASC",
                libsql::params![entry_id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;
        let mut tags = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            tags.push(self.row_to_tag(row)?);
        }
        Ok(tags)
    }

    /// Load tags for multiple entries in one query. Returns map of entry_id -> tags.
    pub async fn get_tags_for_entries(&self, entry_ids: &[String]) -> Result<HashMap<String, Vec<Tag>>> {
        if entry_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        // Build IN clause with escaped ids (same pattern as add_tags in this codebase)
        let escaped_ids: Vec<String> = entry_ids
            .iter()
            .map(|id| format!("'{}'", id.replace('\'', "''")))
            .collect();
        let in_clause = escaped_ids.join(", ");

        let query = format!(
            "SELECT et.entry_id, t.id, t.name, t.created_at, t.updated_at, t.deleted_at,
                    t._sync_id, t._updated_at, t._deleted, t._extra
             FROM entry_tags et
             INNER JOIN tags t ON t.id = et.tag_id
             WHERE et.entry_id IN ({}) AND t.deleted_at IS NULL
             ORDER BY et.entry_id, t.name ASC",
            in_clause
        );

        let mut rows = conn
            .query(&query, libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut map: HashMap<String, Vec<Tag>> = HashMap::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            let entry_id: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
            let tag = self.row_to_tag_from_offset(row, 1)?;
            map.entry(entry_id).or_default().push(tag);
        }

        Ok(map)
    }

    fn row_to_tag(&self, row: libsql::Row) -> Result<Tag> {
        self.row_to_tag_from_offset(row, 0)
    }

    /// Build Tag from row starting at column `offset` (0 = tag-only row, 1 = after entry_id).
    fn row_to_tag_from_offset(&self, row: libsql::Row, offset: usize) -> Result<Tag> {
        let o = offset as i32;
        let id: String = row.get(o).map_err(|e| AppError::LibSQL(e))?;
        let name: String = row.get(o + 1).map_err(|e| AppError::LibSQL(e))?;
        let created_at_str: String = row.get(o + 2).map_err(|e| AppError::LibSQL(e))?;
        let updated_at_str: String = row.get(o + 3).map_err(|e| AppError::LibSQL(e))?;
        let deleted_at_str: Option<String> = row.get(o + 4).map_err(|e| AppError::LibSQL(e))?;
        let _sync_id: Option<String> = row.get(o + 5).ok();
        let _updated_at: Option<i64> = row.get(o + 6).ok();
        let _deleted: i64 = row.get(o + 7).unwrap_or(0);
        let _extra: Option<serde_json::Value> = row
            .get::<Option<String>>(o + 8)
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str(&s).ok());
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid tag created_at: {}", e)))?
            .with_timezone(&Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid tag updated_at: {}", e)))?
            .with_timezone(&Utc);
        let deleted_at = deleted_at_str
            .map(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .flatten()
            .map(|dt| dt.with_timezone(&Utc));
        Ok(Tag {
            id,
            name,
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
