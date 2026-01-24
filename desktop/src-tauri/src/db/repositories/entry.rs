use crate::db::models::Entry;
use crate::error::{AppError, Result};
use crate::utils::generate_id;
use chrono::Utc;
use libsql::Database;
use std::sync::Arc;

pub struct EntryRepository {
    database: Arc<Database>,
}

impl EntryRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Get all entries (non-deleted)
    pub async fn find_all(&self) -> Result<Vec<Entry>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT id, document, created_at, is_pinned, is_archived, is_deleted, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM entries 
                 WHERE is_deleted = 0 
                 ORDER BY created_at ASC",
                libsql::params![],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut entries = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            entries.push(self.row_to_entry(row)?);
        }

        Ok(entries)
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
            Ok(Some(self.row_to_entry(row)?))
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
    pub async fn remove_tags(&self, entry_id: &str, tag_id: String) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Verify entry exists
        let entry = self.find_by_id(entry_id).await?;
        if entry.is_none() {
            return Err(AppError::NotFound(format!("Entry {} not found", entry_id)));
        }

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
            _sync_id,
            _updated_at,
            _deleted: _deleted != 0,
            _extra,
        })
    }
}
