use crate::db::models::{SubTask, Task, TaskWithSubtasks};
use std::collections::HashMap;
use crate::error::{AppError, Result};
use crate::utils::generate_id;
use chrono::Utc;
use libsql::Database;
use std::sync::Arc;

pub struct TaskRepository {
    database: Arc<Database>,
}

impl TaskRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Create a new task
    pub async fn create(
        &self,
        title: String,
        description: Option<String>,
        due_date: Option<chrono::DateTime<Utc>>,
        goal_id: Option<String>,
        goal_instance_id: Option<String>,
    ) -> Result<Task> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let id = generate_id("task");
        let now = Utc::now();
        let created_at_str = now.to_rfc3339();
        let updated_at_str = now.to_rfc3339();
        let due_date_str = due_date.map(|d| d.to_rfc3339());

        let now_ms = now.timestamp_millis();
        conn.execute(
            "INSERT INTO tasks (id, title, description, is_completed, due_date, goal_id, goal_instance_id, created_at, updated_at, _sync_id, _updated_at, _deleted, _extra) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, '{}')",
            libsql::params![
                id.clone(),
                title.clone(),
                description.as_ref().map(|s| s.as_str()),
                if false { 1 } else { 0 }, // is_completed
                due_date_str.as_ref().map(|s| s.as_str()),
                goal_id.as_ref().map(|s| s.as_str()),
                goal_instance_id.as_ref().map(|s| s.as_str()),
                created_at_str,
                updated_at_str,
                id.clone(),
                now_ms,
            ],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(Task {
            id: id.clone(),
            title,
            description,
            is_completed: false,
            due_date,
            goal_instance_id,
            goal_id,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            _sync_id: Some(id),
            _updated_at: Some(now_ms),
            _deleted: false,
            _extra: None,
        })
    }

    /// Get inbox tasks (tasks not attached to any goal)
    /// If limit and cursor are both None, returns all inbox tasks (bypass pagination)
    /// Otherwise returns paginated results with cursor-based pagination
    pub async fn find_inbox(
        &self,
        limit: Option<u32>,
        cursor: Option<String>,
    ) -> Result<(Vec<Task>, Option<String>, bool)> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Bypass mode: return all results
        if limit.is_none() && cursor.is_none() {
            let mut rows = conn
                .query(
                    "SELECT id, title, description, is_completed, due_date, goal_instance_id, goal_id, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                     FROM tasks 
                     WHERE goal_id IS NULL AND deleted_at IS NULL 
                     ORDER BY COALESCE(due_date, '') ASC, id ASC",
                    libsql::params![],
                )
                .await
                .map_err(|e| AppError::LibSQL(e))?;

            let mut tasks = Vec::new();
            while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
                tasks.push(self.row_to_task(row)?);
            }

            return Ok((tasks, None, false));
        }

        // Pagination mode - use composite cursor for due_date + id
        let limit_val = limit.unwrap_or(50).min(1000);
        let fetch_limit = limit_val + 1;
        
        let mut rows = if let Some(cursor_val) = cursor {
            use crate::handlers::common::cursor;
            let keys = cursor::decode_composite(&cursor_val)?;
            if keys.len() != 2 {
                return Err(AppError::BadRequest("Invalid cursor format for tasks".to_string()));
            }
            let last_due_date = &keys[0];
            let last_id = &keys[1];
            
            conn.query(
                "SELECT id, title, description, is_completed, due_date, goal_instance_id, goal_id, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM tasks 
                 WHERE goal_id IS NULL AND deleted_at IS NULL 
                 AND (COALESCE(due_date, '') > ?1 OR (COALESCE(due_date, '') = ?1 AND id > ?2))
                 ORDER BY COALESCE(due_date, '') ASC, id ASC
                 LIMIT ?3",
                libsql::params![last_due_date.clone(), last_id.clone(), fetch_limit as i64],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?
        } else {
            conn.query(
                "SELECT id, title, description, is_completed, due_date, goal_instance_id, goal_id, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM tasks 
                 WHERE goal_id IS NULL AND deleted_at IS NULL 
                 ORDER BY COALESCE(due_date, '') ASC, id ASC
                 LIMIT ?1",
                libsql::params![fetch_limit as i64],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?
        };

        let mut tasks = Vec::new();
        let mut has_more = false;
        
        let mut count = 0;
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            if count < limit_val {
                tasks.push(self.row_to_task(row)?);
                count += 1;
            } else {
                has_more = true;
                break;
            }
        }

        let next_cursor = if has_more && !tasks.is_empty() {
            use crate::handlers::common::cursor;
            let last_task = tasks.last().unwrap();
            let due_date_str = last_task.due_date
                .map(|d| d.to_rfc3339())
                .unwrap_or_else(|| "".to_string());
            Some(cursor::encode_composite(&[&due_date_str, &last_task.id]))
        } else {
            None
        };

        Ok((tasks, next_cursor, has_more))
    }

    /// Get overdue tasks
    /// If limit and cursor are both None, returns all overdue tasks (bypass pagination)
    /// Otherwise returns paginated results with cursor-based pagination
    pub async fn find_overdue(
        &self,
        limit: Option<u32>,
        cursor: Option<String>,
    ) -> Result<(Vec<Task>, Option<String>, bool)> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        // Bypass mode: return all results
        if limit.is_none() && cursor.is_none() {
            let mut rows = conn
                .query(
                    "SELECT id, title, description, is_completed, due_date, goal_instance_id, goal_id, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                     FROM tasks 
                     WHERE due_date < ?1 AND is_completed = 0 AND deleted_at IS NULL 
                     ORDER BY COALESCE(due_date, '') ASC, id ASC",
                    libsql::params![now_str],
                )
                .await
                .map_err(|e| AppError::LibSQL(e))?;

            let mut tasks = Vec::new();
            while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
                tasks.push(self.row_to_task(row)?);
            }

            return Ok((tasks, None, false));
        }

        // Pagination mode - use composite cursor for due_date + id
        let limit_val = limit.unwrap_or(50).min(1000);
        let fetch_limit = limit_val + 1;
        
        let mut rows = if let Some(cursor_val) = cursor {
            use crate::handlers::common::cursor;
            let keys = cursor::decode_composite(&cursor_val)?;
            if keys.len() != 2 {
                return Err(AppError::BadRequest("Invalid cursor format for tasks".to_string()));
            }
            let last_due_date = &keys[0];
            let last_id = &keys[1];
            
            conn.query(
                "SELECT id, title, description, is_completed, due_date, goal_instance_id, goal_id, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM tasks 
                 WHERE due_date < ?1 AND is_completed = 0 AND deleted_at IS NULL 
                 AND (COALESCE(due_date, '') > ?2 OR (COALESCE(due_date, '') = ?2 AND id > ?3))
                 ORDER BY COALESCE(due_date, '') ASC, id ASC
                 LIMIT ?4",
                libsql::params![now_str, last_due_date.clone(), last_id.clone(), fetch_limit as i64],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?
        } else {
            conn.query(
                "SELECT id, title, description, is_completed, due_date, goal_instance_id, goal_id, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM tasks 
                 WHERE due_date < ?1 AND is_completed = 0 AND deleted_at IS NULL 
                 ORDER BY COALESCE(due_date, '') ASC, id ASC
                 LIMIT ?2",
                libsql::params![now_str, fetch_limit as i64],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?
        };

        let mut tasks = Vec::new();
        let mut has_more = false;
        
        let mut count = 0;
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            if count < limit_val {
                tasks.push(self.row_to_task(row)?);
                count += 1;
            } else {
                has_more = true;
                break;
            }
        }

        let next_cursor = if has_more && !tasks.is_empty() {
            use crate::handlers::common::cursor;
            let last_task = tasks.last().unwrap();
            let due_date_str = last_task.due_date
                .map(|d| d.to_rfc3339())
                .unwrap_or_else(|| "".to_string());
            Some(cursor::encode_composite(&[&due_date_str, &last_task.id]))
        } else {
            None
        };

        Ok((tasks, next_cursor, has_more))
    }

    /// Get task by ID
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Task>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT id, title, description, is_completed, due_date, goal_instance_id, goal_id, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM tasks 
                 WHERE id = ?1 AND deleted_at IS NULL",
                libsql::params![id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        if let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            Ok(Some(self.row_to_task(row)?))
        } else {
            Ok(None)
        }
    }

    /// Update a task
    pub async fn update(
        &self,
        id: &str,
        title: Option<String>,
        description: Option<Option<String>>,
        due_date: Option<Option<chrono::DateTime<Utc>>>,
        is_completed: Option<bool>,
        goal_id: Option<Option<String>>,
        goal_instance_id: Option<Option<String>>,
        client_updated_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<Task> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Get current task
        let current = self.find_by_id(id).await?;
        let mut task = current.ok_or_else(|| AppError::NotFound(format!("Task {} not found", id)))?;

        // Last-Write-Wins conflict detection
        if let Some(client_time) = client_updated_at {
            if client_time < task.updated_at {
                return Err(AppError::BadRequest(format!(
                    "Conflict: Task was modified by another device. Current updated_at: {}",
                    task.updated_at.to_rfc3339()
                )));
            }
        }

        // Update fields
        if let Some(t) = title {
            task.title = t;
        }
        if let Some(d) = description {
            task.description = d;
        }
        if let Some(dd) = due_date {
            task.due_date = dd;
        }
        if let Some(ic) = is_completed {
            task.is_completed = ic;
        }
        if let Some(gid) = goal_id {
            task.goal_id = gid.clone();
            task.goal_instance_id = goal_instance_id.flatten();
        }
        task.updated_at = Utc::now();
        let now_ms = task.updated_at.timestamp_millis();
        task._updated_at = Some(now_ms);

        let updated_at_str = task.updated_at.to_rfc3339();
        let due_date_str = task.due_date.map(|d| d.to_rfc3339());

        conn.execute(
            "UPDATE tasks 
             SET title = ?1, description = ?2, is_completed = ?3, due_date = ?4, goal_id = ?5, goal_instance_id = ?6, updated_at = ?7, _updated_at = ?8 
             WHERE id = ?9",
            libsql::params![
                task.title.clone(),
                task.description.as_ref().map(|s| s.as_str()),
                if task.is_completed { 1 } else { 0 },
                due_date_str.as_ref().map(|s| s.as_str()),
                task.goal_id.as_ref().map(|s| s.as_str()),
                task.goal_instance_id.as_ref().map(|s| s.as_str()),
                updated_at_str,
                now_ms,
                id
            ],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(task)
    }

    /// Delete a task (soft delete)
    pub async fn delete(&self, id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Check if task exists
        let task = self.find_by_id(id).await?;
        if task.is_none() {
            return Err(AppError::NotFound(format!("Task {} not found", id)));
        }

        let now = Utc::now();
        let updated_at_str = now.to_rfc3339();
        let deleted_at_str = now.to_rfc3339();
        let now_ms = now.timestamp_millis();

        conn.execute(
            "UPDATE tasks SET deleted_at = ?1, updated_at = ?2, _updated_at = ?3, _deleted = 1 WHERE id = ?4",
            libsql::params![deleted_at_str, updated_at_str, now_ms, id],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Get subtasks for a task
    pub async fn find_subtasks(&self, task_id: &str) -> Result<Vec<SubTask>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Verify task exists
        let task = self.find_by_id(task_id).await?;
        if task.is_none() {
            return Err(AppError::NotFound(format!("Task {} not found", task_id)));
        }

        let mut rows = conn
            .query(
                "SELECT id, title, is_completed, task_id, order_index, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM subtasks 
                 WHERE task_id = ?1 AND deleted_at IS NULL 
                 ORDER BY order_index ASC",
                libsql::params![task_id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut subtasks = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            subtasks.push(self.row_to_subtask(row)?);
        }

        Ok(subtasks)
    }

    /// Get subtasks for multiple tasks efficiently (batch query)
    pub async fn find_subtasks_for_tasks(&self, task_ids: &[String]) -> Result<HashMap<String, Vec<SubTask>>> {
        if task_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Build IN clause with escaped IDs
        let escaped_ids: Vec<String> = task_ids
            .iter()
            .map(|id| format!("'{}'", id.replace("'", "''")))
            .collect();
        let query = format!(
            "SELECT id, title, is_completed, task_id, order_index, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
             FROM subtasks 
             WHERE task_id IN ({}) AND deleted_at IS NULL 
             ORDER BY task_id, order_index ASC",
            escaped_ids.join(", ")
        );

        let mut rows = conn
            .query(&query, libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut subtasks_map: HashMap<String, Vec<SubTask>> = HashMap::new();
        
        // Initialize all task_ids with empty vectors
        for task_id in task_ids {
            subtasks_map.insert(task_id.clone(), Vec::new());
        }

        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            let subtask = self.row_to_subtask(row)?;
            subtasks_map
                .entry(subtask.task_id.clone())
                .or_insert_with(Vec::new)
                .push(subtask);
        }

        Ok(subtasks_map)
    }

    /// Convert tasks to tasks with subtasks
    pub async fn with_subtasks(&self, tasks: Vec<Task>) -> Result<Vec<TaskWithSubtasks>> {
        let task_ids: Vec<String> = tasks.iter().map(|t| t.id.clone()).collect();
        let subtasks_map = self.find_subtasks_for_tasks(&task_ids).await?;

        let tasks_with_subtasks = tasks
            .into_iter()
            .map(|task| {
                let subtasks = subtasks_map
                    .get(&task.id)
                    .cloned()
                    .unwrap_or_default();
                TaskWithSubtasks { task, subtasks }
            })
            .collect();

        Ok(tasks_with_subtasks)
    }

    /// Create a subtask
    pub async fn create_subtask(&self, task_id: &str, title: String) -> Result<SubTask> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Verify task exists
        let task = self.find_by_id(task_id).await?;
        if task.is_none() {
            return Err(AppError::NotFound(format!("Task {} not found", task_id)));
        }

        // Get max order index
        let mut rows = conn
            .query(
                "SELECT COALESCE(MAX(order_index), -1) FROM subtasks WHERE task_id = ?1",
                libsql::params![task_id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let max_order = if let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            row.get::<i64>(0).map_err(|e| AppError::LibSQL(e))? as i32
        } else {
            -1
        };

        let id = generate_id("subtask");
        let now = Utc::now();
        let created_at_str = now.to_rfc3339();
        let updated_at_str = now.to_rfc3339();
        let order_index = max_order + 1;

        let now_ms = now.timestamp_millis();
        conn.execute(
            "INSERT INTO subtasks (id, title, is_completed, task_id, order_index, created_at, updated_at, _sync_id, _updated_at, _deleted, _extra) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, '{}')",
            libsql::params![
                id.clone(),
                title.clone(),
                if false { 1 } else { 0 }, // is_completed
                task_id,
                order_index,
                created_at_str,
                updated_at_str,
                id.clone(),
                now_ms,
            ],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(SubTask {
            id: id.clone(),
            title,
            is_completed: false,
            task_id: task_id.to_string(),
            order_index,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            _sync_id: Some(id),
            _updated_at: Some(now_ms),
            _deleted: false,
            _extra: None,
        })
    }

    /// Update a subtask
    pub async fn update_subtask(
        &self,
        task_id: &str,
        subtask_id: &str,
        title: Option<String>,
        is_completed: Option<bool>,
    ) -> Result<SubTask> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Get current subtask
        let mut rows = conn
            .query(
                "SELECT id, title, is_completed, task_id, order_index, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM subtasks 
                 WHERE id = ?1 AND task_id = ?2 AND deleted_at IS NULL",
                libsql::params![subtask_id, task_id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut subtask = if let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            self.row_to_subtask(row)?
        } else {
            return Err(AppError::NotFound(format!("Subtask {} not found", subtask_id)));
        };

        // Update fields
        if let Some(t) = title {
            subtask.title = t;
        }
        if let Some(ic) = is_completed {
            subtask.is_completed = ic;
        }
        subtask.updated_at = Utc::now();
        let now_ms = subtask.updated_at.timestamp_millis();
        subtask._updated_at = Some(now_ms);

        let updated_at_str = subtask.updated_at.to_rfc3339();

        conn.execute(
            "UPDATE subtasks SET title = ?1, is_completed = ?2, updated_at = ?3, _updated_at = ?4 WHERE id = ?5",
            libsql::params![
                subtask.title.clone(),
                if subtask.is_completed { 1 } else { 0 },
                updated_at_str,
                now_ms,
                subtask_id
            ],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(subtask)
    }

    /// Delete a subtask
    pub async fn delete_subtask(&self, task_id: &str, subtask_id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Check if subtask exists
        let mut rows = conn
            .query(
                "SELECT id FROM subtasks WHERE id = ?1 AND task_id = ?2 AND deleted_at IS NULL",
                libsql::params![subtask_id, task_id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        if rows.next().await.map_err(|e| AppError::LibSQL(e))?.is_none() {
            return Err(AppError::NotFound(format!("Subtask {} not found", subtask_id)));
        }

        let now = Utc::now();
        let updated_at_str = now.to_rfc3339();
        let deleted_at_str = now.to_rfc3339();
        let now_ms = now.timestamp_millis();

        conn.execute(
            "UPDATE subtasks SET deleted_at = ?1, updated_at = ?2, _updated_at = ?3, _deleted = 1 WHERE id = ?4",
            libsql::params![deleted_at_str, updated_at_str, now_ms, subtask_id],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Reorder subtasks
    pub async fn reorder_subtasks(&self, task_id: &str, subtask_ids: Vec<String>) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Verify task exists
        let task = self.find_by_id(task_id).await?;
        if task.is_none() {
            return Err(AppError::NotFound(format!("Task {} not found", task_id)));
        }

        let now_ms = Utc::now().timestamp_millis();
        conn.execute("BEGIN TRANSACTION", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        for (index, subtask_id) in subtask_ids.iter().enumerate() {
            conn.execute(
                "UPDATE subtasks SET order_index = ?1, _updated_at = ?2 WHERE id = ?3 AND task_id = ?4",
                libsql::params![index as i32, now_ms, subtask_id.as_str(), task_id],
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

    /// Add tags to a task
    pub async fn add_tags(&self, task_id: &str, tag_ids: Vec<String>) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Verify task exists
        let task = self.find_by_id(task_id).await?;
        if task.is_none() {
            return Err(AppError::NotFound(format!("Task {} not found", task_id)));
        }

        // Verify tags exist
        if tag_ids.is_empty() {
            return Ok(());
        }

        // Verify tags exist - use single query with IN clause to avoid N+1
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
        let now_ms = Utc::now().timestamp_millis();
        conn.execute("BEGIN TRANSACTION", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        for tag_id in tag_ids {
            let sync_id = format!("{}|{}", task_id, tag_id);
            conn.execute(
                "INSERT OR IGNORE INTO task_tags (task_id, tag_id, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, 0, '{}')",
                libsql::params![task_id, tag_id, sync_id, now_ms],
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

    /// Remove tags from a task
    pub async fn remove_tags(&self, task_id: &str, tag_ids: Vec<String>) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Verify task exists
        let task = self.find_by_id(task_id).await?;
        if task.is_none() {
            return Err(AppError::NotFound(format!("Task {} not found", task_id)));
        }

        if tag_ids.is_empty() {
            return Ok(());
        }

        conn.execute("BEGIN TRANSACTION", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        for tag_id in &tag_ids {
            conn.execute(
                "DELETE FROM task_tags WHERE task_id = ?1 AND tag_id = ?2",
                libsql::params![task_id, tag_id.as_str()],
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

    /// Add goal to a task - gets or creates current goal instance
    pub async fn add_goal(&self, _task_id: &str, goal_id: &str) -> Result<Option<String>> {
        // Use GoalRepository to get or create current instance
        use crate::db::repositories::goal::GoalRepository;
        let goal_repo = GoalRepository::new(self.database.clone());
        let instance = goal_repo.get_or_create_current_instance(goal_id).await?;
        Ok(Some(instance.id))
    }

    /// Remove goal from a task
    pub async fn remove_goal(&self, task_id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Verify task exists
        let task = self.find_by_id(task_id).await?;
        if task.is_none() {
            return Err(AppError::NotFound(format!("Task {} not found", task_id)));
        }

        let now = Utc::now();
        let updated_at_str = now.to_rfc3339();
        let now_ms = now.timestamp_millis();

        conn.execute(
            "UPDATE tasks SET goal_id = NULL, goal_instance_id = NULL, updated_at = ?1, _updated_at = ?2 WHERE id = ?3",
            libsql::params![updated_at_str, now_ms, task_id],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Helper to convert database row to Task
    fn row_to_task(&self, row: libsql::Row) -> Result<Task> {
        let id: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
        let title: String = row.get(1).map_err(|e| AppError::LibSQL(e))?;
        let description: Option<String> = row.get(2).map_err(|e| AppError::LibSQL(e))?;
        let is_completed: i64 = row.get(3).map_err(|e| AppError::LibSQL(e))?;
        let due_date_str: Option<String> = row.get(4).map_err(|e| AppError::LibSQL(e))?;
        let goal_instance_id: Option<String> = row.get(5).map_err(|e| AppError::LibSQL(e))?;
        let goal_id: Option<String> = row.get(6).map_err(|e| AppError::LibSQL(e))?;
        let created_at_str: String = row.get(7).map_err(|e| AppError::LibSQL(e))?;
        let updated_at_str: String = row.get(8).map_err(|e| AppError::LibSQL(e))?;
        let deleted_at_str: Option<String> = row.get(9).map_err(|e| AppError::LibSQL(e))?;
        let _sync_id: Option<String> = row.get(10).ok();
        let _updated_at: Option<i64> = row.get(11).ok();
        let _deleted: i64 = row.get(12).unwrap_or(0);
        let _extra: Option<serde_json::Value> = row.get::<Option<String>>(13).ok().flatten().and_then(|s| serde_json::from_str(&s).ok());

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid updated_at: {}", e)))?
            .with_timezone(&Utc);
        let due_date = due_date_str
            .map(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .flatten()
            .map(|dt| dt.with_timezone(&Utc));
        let deleted_at = deleted_at_str
            .map(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .flatten()
            .map(|dt| dt.with_timezone(&Utc));

        Ok(Task {
            id,
            title,
            description,
            is_completed: is_completed != 0,
            due_date,
            goal_instance_id,
            goal_id,
            created_at,
            updated_at,
            deleted_at,
            _sync_id,
            _updated_at,
            _deleted: _deleted != 0,
            _extra,
        })
    }

    /// Helper to convert database row to SubTask
    fn row_to_subtask(&self, row: libsql::Row) -> Result<SubTask> {
        let id: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
        let title: String = row.get(1).map_err(|e| AppError::LibSQL(e))?;
        let is_completed: i64 = row.get(2).map_err(|e| AppError::LibSQL(e))?;
        let task_id: String = row.get(3).map_err(|e| AppError::LibSQL(e))?;
        let order_index: i64 = row.get(4).map_err(|e| AppError::LibSQL(e))?;
        let created_at_str: String = row.get(5).map_err(|e| AppError::LibSQL(e))?;
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

        Ok(SubTask {
            id,
            title,
            is_completed: is_completed != 0,
            task_id,
            order_index: order_index as i32,
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
