use crate::db::models::{Goal, SubTask, Tag, Task, TaskWithSubtasks};
use crate::error::{AppError, Result};
use crate::utils::{generate_id, record_rust_timing};
use chrono::Utc;
use libsql::Database;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

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
            subtasks: None,
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
        let repository_started = Instant::now();
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        let connect_ms = repository_started.elapsed().as_secs_f64() * 1000.0;

        // Bypass mode: return all results
        if limit.is_none() && cursor.is_none() {
            let query_started = Instant::now();
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
            let query_ms = query_started.elapsed().as_secs_f64() * 1000.0;

            record_rust_timing(
                "rust-repository",
                "TaskRepository.find_inbox",
                repository_started.elapsed(),
                json!({
                    "resource_type": "task",
                    "mode": "all",
                    "result_count": tasks.len(),
                    "connect_ms": (connect_ms * 10.0).round() / 10.0,
                    "query_rows_ms": (query_ms * 10.0).round() / 10.0,
                }),
            );

            return Ok((tasks, None, false));
        }

        // Pagination mode - use composite cursor for due_date + id
        let limit_val = limit.unwrap_or(50).min(1000);
        let fetch_limit = limit_val + 1;

        let has_cursor = cursor.is_some();
        let query_started = Instant::now();
        let mut rows = if let Some(cursor_val) = cursor {
            use crate::commands::common::cursor;
            let keys = cursor::decode_composite(&cursor_val)?;
            if keys.len() != 2 {
                return Err(AppError::BadRequest(
                    "Invalid cursor format for tasks".to_string(),
                ));
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
        let query_ms = query_started.elapsed().as_secs_f64() * 1000.0;

        let next_cursor = if has_more && !tasks.is_empty() {
            use crate::commands::common::cursor;
            let last_task = tasks.last().unwrap();
            let due_date_str = last_task
                .due_date
                .map(|d| d.to_rfc3339())
                .unwrap_or_else(|| "".to_string());
            Some(cursor::encode_composite(&[&due_date_str, &last_task.id]))
        } else {
            None
        };

        record_rust_timing(
            "rust-repository",
            "TaskRepository.find_inbox",
            repository_started.elapsed(),
            json!({
                "resource_type": "task",
                "mode": "paginated",
                "result_count": tasks.len(),
                "limit": limit_val,
                "cursor_present": has_cursor,
                "has_more": has_more,
                "connect_ms": (connect_ms * 10.0).round() / 10.0,
                "query_rows_ms": (query_ms * 10.0).round() / 10.0,
            }),
        );

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
        let repository_started = Instant::now();
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        let connect_ms = repository_started.elapsed().as_secs_f64() * 1000.0;

        let now = Utc::now();
        let now_str = now.to_rfc3339();

        // Bypass mode: return all results
        if limit.is_none() && cursor.is_none() {
            let query_started = Instant::now();
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
            let query_ms = query_started.elapsed().as_secs_f64() * 1000.0;

            record_rust_timing(
                "rust-repository",
                "TaskRepository.find_overdue",
                repository_started.elapsed(),
                json!({
                    "resource_type": "task",
                    "mode": "all",
                    "result_count": tasks.len(),
                    "connect_ms": (connect_ms * 10.0).round() / 10.0,
                    "query_rows_ms": (query_ms * 10.0).round() / 10.0,
                }),
            );

            return Ok((tasks, None, false));
        }

        // Pagination mode - use composite cursor for due_date + id
        let limit_val = limit.unwrap_or(50).min(1000);
        let fetch_limit = limit_val + 1;

        let has_cursor = cursor.is_some();
        let query_started = Instant::now();
        let mut rows = if let Some(cursor_val) = cursor {
            use crate::commands::common::cursor;
            let keys = cursor::decode_composite(&cursor_val)?;
            if keys.len() != 2 {
                return Err(AppError::BadRequest(
                    "Invalid cursor format for tasks".to_string(),
                ));
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
        let query_ms = query_started.elapsed().as_secs_f64() * 1000.0;

        let next_cursor = if has_more && !tasks.is_empty() {
            use crate::commands::common::cursor;
            let last_task = tasks.last().unwrap();
            let due_date_str = last_task
                .due_date
                .map(|d| d.to_rfc3339())
                .unwrap_or_else(|| "".to_string());
            Some(cursor::encode_composite(&[&due_date_str, &last_task.id]))
        } else {
            None
        };

        record_rust_timing(
            "rust-repository",
            "TaskRepository.find_overdue",
            repository_started.elapsed(),
            json!({
                "resource_type": "task",
                "mode": "paginated",
                "result_count": tasks.len(),
                "limit": limit_val,
                "cursor_present": has_cursor,
                "has_more": has_more,
                "connect_ms": (connect_ms * 10.0).round() / 10.0,
                "query_rows_ms": (query_ms * 10.0).round() / 10.0,
            }),
        );

        Ok((tasks, next_cursor, has_more))
    }

    /// Get task by ID
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Task>> {
        let repository_started = Instant::now();
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        let connect_ms = repository_started.elapsed().as_secs_f64() * 1000.0;

        let query_started = Instant::now();
        let mut rows = conn
            .query(
                "SELECT id, title, description, is_completed, due_date, goal_instance_id, goal_id, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM tasks 
                 WHERE id = ?1 AND deleted_at IS NULL",
                libsql::params![id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;
        let query_ms = query_started.elapsed().as_secs_f64() * 1000.0;

        if let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            record_rust_timing(
                "rust-repository",
                "TaskRepository.find_by_id",
                repository_started.elapsed(),
                json!({
                    "resource_type": "task",
                    "resource_id": id,
                    "found": true,
                    "connect_ms": (connect_ms * 10.0).round() / 10.0,
                    "query_ms": (query_ms * 10.0).round() / 10.0,
                }),
            );
            Ok(Some(self.row_to_task(row)?))
        } else {
            record_rust_timing(
                "rust-repository",
                "TaskRepository.find_by_id",
                repository_started.elapsed(),
                json!({
                    "resource_type": "task",
                    "resource_id": id,
                    "found": false,
                    "connect_ms": (connect_ms * 10.0).round() / 10.0,
                    "query_ms": (query_ms * 10.0).round() / 10.0,
                }),
            );
            Ok(None)
        }
    }

    /// Get tasks for a goal instance (for goal view)
    pub async fn find_by_goal_instance_id(&self, goal_instance_id: &str) -> Result<Vec<Task>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        let mut rows = conn
            .query(
                "SELECT id, title, description, is_completed, due_date, goal_instance_id, goal_id, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM tasks 
                 WHERE goal_instance_id = ?1 AND deleted_at IS NULL 
                 ORDER BY COALESCE(due_date, '') ASC, id ASC",
                libsql::params![goal_instance_id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut tasks = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            tasks.push(self.row_to_task(row)?);
        }
        Ok(tasks)
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
        let mut task =
            current.ok_or_else(|| AppError::NotFound(format!("Task {} not found", id)))?;

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

    /// Restore a soft-deleted task.
    pub async fn restore(&self, id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        let mut rows = conn
            .query(
                "SELECT id FROM tasks WHERE id = ?1 AND deleted_at IS NOT NULL",
                libsql::params![id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        if rows
            .next()
            .await
            .map_err(|e| AppError::LibSQL(e))?
            .is_none()
        {
            return Err(AppError::NotFound(format!("Deleted task {} not found", id)));
        }

        let now = Utc::now();
        let updated_at_str = now.to_rfc3339();
        let now_ms = now.timestamp_millis();

        conn.execute(
            "UPDATE tasks SET deleted_at = NULL, updated_at = ?1, _updated_at = ?2, _deleted = 0 WHERE id = ?3",
            libsql::params![updated_at_str, now_ms, id],
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
    pub async fn find_subtasks_for_tasks(
        &self,
        task_ids: &[String],
    ) -> Result<HashMap<String, Vec<SubTask>>> {
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

    /// Load tags for multiple tasks in one query. Returns map of task_id -> tags.
    pub async fn get_tags_for_tasks(
        &self,
        task_ids: &[String],
    ) -> Result<HashMap<String, Vec<Tag>>> {
        if task_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        let escaped: Vec<String> = task_ids
            .iter()
            .map(|id| format!("'{}'", id.replace('\'', "''")))
            .collect();
        let in_clause = escaped.join(", ");
        let query = format!(
            "SELECT tt.task_id, t.id, t.name, t.created_at, t.updated_at, t.deleted_at, t._sync_id, t._updated_at, t._deleted, t._extra
             FROM task_tags tt
             INNER JOIN tags t ON t.id = tt.tag_id
             WHERE tt.task_id IN ({}) AND COALESCE(tt._deleted, 0) = 0 AND t.deleted_at IS NULL
             ORDER BY tt.task_id, t.name ASC",
            in_clause
        );
        let mut rows = conn
            .query(&query, libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;
        let mut map: HashMap<String, Vec<Tag>> = HashMap::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            let task_id: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
            let tag = self.row_to_tag_from_offset(row, 1)?;
            map.entry(task_id).or_default().push(tag);
        }
        Ok(map)
    }

    /// Convert tasks to tasks with subtasks, tags, and goals (bulk load, no N+1).
    pub async fn with_subtasks(&self, tasks: Vec<Task>) -> Result<Vec<TaskWithSubtasks>> {
        let task_ids: Vec<String> = tasks.iter().map(|t| t.id.clone()).collect();
        let (subtasks_map, tags_map, goals_map) = tokio::try_join!(
            self.find_subtasks_for_tasks(&task_ids),
            self.get_tags_for_tasks(&task_ids),
            async {
                let goal_ids: Vec<String> = tasks
                    .iter()
                    .filter_map(|t| t.goal_id.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();
                if goal_ids.is_empty() {
                    Ok(HashMap::new())
                } else {
                    crate::db::GoalRepository::new(self.database.clone())
                        .find_by_ids(&goal_ids)
                        .await
                }
            }
        )?;

        let tasks_with_subtasks = tasks
            .into_iter()
            .map(|task| {
                let subtasks = subtasks_map.get(&task.id).cloned().unwrap_or_default();
                let tags = tags_map.get(&task.id).cloned().unwrap_or_default();
                let goal = task
                    .goal_id
                    .as_ref()
                    .and_then(|gid| goals_map.get(gid).cloned());
                Self::task_to_task_with_subtasks(task, subtasks, tags, goal)
            })
            .collect();

        Ok(tasks_with_subtasks)
    }

    /// Helper to convert Task + subtasks + tags + goal to TaskWithSubtasks
    pub fn task_to_task_with_subtasks(
        task: Task,
        subtasks: Vec<SubTask>,
        tags: Vec<Tag>,
        goal: Option<Goal>,
    ) -> TaskWithSubtasks {
        TaskWithSubtasks {
            id: task.id,
            title: task.title,
            description: task.description,
            is_completed: task.is_completed,
            due_date: task.due_date,
            goal_instance_id: task.goal_instance_id,
            goal_id: task.goal_id,
            created_at: task.created_at,
            updated_at: task.updated_at,
            deleted_at: task.deleted_at,
            subtasks,
            tags,
            goal,
        }
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
            return Err(AppError::NotFound(format!(
                "Subtask {} not found",
                subtask_id
            )));
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

        if rows
            .next()
            .await
            .map_err(|e| AppError::LibSQL(e))?
            .is_none()
        {
            return Err(AppError::NotFound(format!(
                "Subtask {} not found",
                subtask_id
            )));
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
        let query = format!(
            "SELECT id FROM tags WHERE id IN ({})",
            escaped_ids.join(", ")
        );

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

    /// Build Tag from row starting at column `offset` (e.g. 1 after task_id).
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
        let _extra: Option<serde_json::Value> = row
            .get::<Option<String>>(13)
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str(&s).ok());

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
            subtasks: None,
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
        let _extra: Option<serde_json::Value> = row
            .get::<Option<String>>(11)
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str(&s).ok());

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::db::repositories::{GoalRepository, TagRepository};
    use chrono::{DateTime, Duration, Utc};
    use libsql::Builder;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static DB_COUNTER: AtomicU64 = AtomicU64::new(0);

    async fn test_repo() -> (Arc<Database>, TaskRepository, PathBuf) {
        let id = DB_COUNTER.fetch_add(1, Ordering::Relaxed);
        let db_path =
            std::env::temp_dir().join(format!("aether-task-test-{}-{}.db", std::process::id(), id));
        let database = Builder::new_local(&db_path)
            .build()
            .await
            .expect("create test database");
        migrations::run_migrations(&database)
            .await
            .expect("run migrations");
        let database = Arc::new(database);
        let repo = TaskRepository::new(database.clone());
        (database, repo, db_path)
    }

    fn fixed_utc(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .expect("fixed datetime should parse")
            .with_timezone(&Utc)
    }

    fn cleanup_db(path: PathBuf) {
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
    }

    async fn raw_task_deleted_state(database: &Database, task_id: &str) -> (Option<String>, i64) {
        let conn = database.connect().expect("connect to test database");
        let mut rows = conn
            .query(
                "SELECT deleted_at, _deleted FROM tasks WHERE id = ?1",
                libsql::params![task_id],
            )
            .await
            .expect("query raw task deletion state");
        let row = rows
            .next()
            .await
            .expect("read raw task deletion state")
            .expect("task row should exist");
        (
            row.get(0).expect("deleted_at"),
            row.get(1).expect("_deleted"),
        )
    }

    #[tokio::test]
    async fn task_lifecycle_creates_updates_deletes_and_restores() {
        let (database, repo, db_path) = test_repo().await;
        let due_date = fixed_utc("2026-02-03T04:05:06Z");

        let created = repo
            .create(
                "Draft release checklist".to_string(),
                Some("Initial checklist".to_string()),
                None,
                None,
                None,
            )
            .await
            .expect("create task");

        assert_eq!(created.title, "Draft release checklist");
        assert_eq!(created.description.as_deref(), Some("Initial checklist"));
        assert!(!created.is_completed);
        assert_eq!(created.due_date, None);

        let updated = repo
            .update(
                &created.id,
                Some("Finalize release checklist".to_string()),
                Some(Some("Ship blockers only".to_string())),
                Some(Some(due_date)),
                Some(true),
                None,
                None,
                None,
            )
            .await
            .expect("update task");

        assert_eq!(updated.title, "Finalize release checklist");
        assert_eq!(updated.description.as_deref(), Some("Ship blockers only"));
        assert_eq!(updated.due_date, Some(due_date));
        assert!(updated.is_completed);

        repo.delete(&created.id).await.expect("delete task");
        assert!(repo
            .find_by_id(&created.id)
            .await
            .expect("find deleted task")
            .is_none());
        let (deleted_at, deleted_flag) = raw_task_deleted_state(&database, &created.id).await;
        assert!(deleted_at.is_some());
        assert_eq!(deleted_flag, 1);

        repo.restore(&created.id).await.expect("restore task");
        let restored = repo
            .find_by_id(&created.id)
            .await
            .expect("find restored task")
            .expect("restored task should be visible");
        assert_eq!(restored.title, "Finalize release checklist");
        let (deleted_at, deleted_flag) = raw_task_deleted_state(&database, &created.id).await;
        assert_eq!(deleted_at, None);
        assert_eq!(deleted_flag, 0);

        cleanup_db(db_path);
    }

    #[tokio::test]
    async fn task_child_resources_cover_subtasks_and_goal_relationships() {
        let (database, repo, db_path) = test_repo().await;
        let goal_repo = GoalRepository::new(database.clone());
        let goal = goal_repo
            .create(
                "Release readiness".to_string(),
                Some("Tasks required before tagging".to_string()),
                true,
                None,
                None,
                None,
                None,
                "UTC".to_string(),
            )
            .await
            .expect("create goal");
        let task = repo
            .create("Prepare release".to_string(), None, None, None, None)
            .await
            .expect("create task");

        let goal_instance_id = repo
            .add_goal(&task.id, &goal.id)
            .await
            .expect("create current goal instance");
        let assigned = repo
            .update(
                &task.id,
                None,
                None,
                None,
                None,
                Some(Some(goal.id.clone())),
                Some(goal_instance_id.clone()),
                None,
            )
            .await
            .expect("assign goal");
        assert_eq!(assigned.goal_id.as_deref(), Some(goal.id.as_str()));
        assert_eq!(assigned.goal_instance_id, goal_instance_id);

        let first = repo
            .create_subtask(&task.id, "Audit children".to_string())
            .await
            .expect("create first subtask");
        let second = repo
            .create_subtask(&task.id, "Run task tests".to_string())
            .await
            .expect("create second subtask");
        let third = repo
            .create_subtask(&task.id, "Check release gate".to_string())
            .await
            .expect("create third subtask");

        assert_eq!(first.order_index, 0);
        assert_eq!(second.order_index, 1);
        assert_eq!(third.order_index, 2);

        let updated_second = repo
            .update_subtask(
                &task.id,
                &second.id,
                Some("Run child task tests".to_string()),
                Some(true),
            )
            .await
            .expect("update subtask");
        assert_eq!(updated_second.title, "Run child task tests");
        assert!(updated_second.is_completed);

        repo.reorder_subtasks(
            &task.id,
            vec![third.id.clone(), first.id.clone(), second.id.clone()],
        )
        .await
        .expect("reorder subtasks");
        let reordered = repo.find_subtasks(&task.id).await.expect("find subtasks");
        assert_eq!(
            reordered
                .iter()
                .map(|subtask| (subtask.title.as_str(), subtask.order_index))
                .collect::<Vec<_>>(),
            vec![
                ("Check release gate", 0),
                ("Audit children", 1),
                ("Run child task tests", 2),
            ]
        );

        repo.delete_subtask(&task.id, &first.id)
            .await
            .expect("delete subtask");
        let remaining = repo.find_subtasks(&task.id).await.expect("find remaining");
        assert_eq!(
            remaining
                .iter()
                .map(|subtask| subtask.id.as_str())
                .collect::<Vec<_>>(),
            vec![third.id.as_str(), second.id.as_str()]
        );

        let hydrated = repo
            .with_subtasks(vec![repo.find_by_id(&task.id).await.unwrap().unwrap()])
            .await
            .expect("hydrate task")
            .pop()
            .expect("hydrated task");
        assert_eq!(
            hydrated.goal.as_ref().map(|g| g.name.as_str()),
            Some("Release readiness")
        );
        assert_eq!(hydrated.subtasks.len(), 2);

        repo.remove_goal(&task.id).await.expect("remove goal");
        let unassigned = repo
            .find_by_id(&task.id)
            .await
            .expect("find task")
            .expect("task should exist");
        assert_eq!(unassigned.goal_id, None);
        assert_eq!(unassigned.goal_instance_id, None);

        cleanup_db(db_path);
    }

    #[tokio::test]
    async fn task_inbox_overdue_and_tags_stay_scoped_to_visible_tasks() {
        let (database, repo, db_path) = test_repo().await;
        let tag_repo = TagRepository::new(database.clone());
        let goal_repo = GoalRepository::new(database.clone());
        let past_due = Utc::now() - Duration::days(2);
        let future_due = Utc::now() + Duration::days(2);

        let inbox_overdue = repo
            .create(
                "Answer overdue inbox task".to_string(),
                None,
                Some(past_due),
                None,
                None,
            )
            .await
            .expect("create overdue inbox task");
        let inbox_future = repo
            .create(
                "Plan future inbox task".to_string(),
                None,
                Some(future_due),
                None,
                None,
            )
            .await
            .expect("create future inbox task");
        let completed_overdue = repo
            .create(
                "Completed overdue task".to_string(),
                None,
                Some(past_due),
                None,
                None,
            )
            .await
            .expect("create completed overdue task");
        repo.update(
            &completed_overdue.id,
            None,
            None,
            None,
            Some(true),
            None,
            None,
            None,
        )
        .await
        .expect("complete overdue task");

        let goal = goal_repo
            .create(
                "Scoped goal".to_string(),
                None,
                true,
                None,
                None,
                None,
                None,
                "UTC".to_string(),
            )
            .await
            .expect("create goal");
        let goal_instance_id = repo
            .add_goal(&inbox_future.id, &goal.id)
            .await
            .expect("create goal instance");
        repo.update(
            &inbox_future.id,
            None,
            None,
            None,
            None,
            Some(Some(goal.id.clone())),
            Some(goal_instance_id),
            None,
        )
        .await
        .expect("assign future task to goal");

        let (all_inbox, _, _) = repo.find_inbox(None, None).await.expect("find all inbox");
        let mut all_inbox_ids = all_inbox
            .iter()
            .map(|task| task.id.as_str())
            .collect::<Vec<_>>();
        all_inbox_ids.sort_unstable();
        let mut expected_inbox_ids = vec![inbox_overdue.id.as_str(), completed_overdue.id.as_str()];
        expected_inbox_ids.sort_unstable();
        assert_eq!(all_inbox_ids, expected_inbox_ids);
        let (paged_inbox, _, _) = repo
            .find_inbox(Some(10), None)
            .await
            .expect("find inbox page");
        let mut paged_inbox_ids = paged_inbox
            .iter()
            .map(|task| task.id.as_str())
            .collect::<Vec<_>>();
        paged_inbox_ids.sort_unstable();
        assert_eq!(paged_inbox_ids, expected_inbox_ids);

        let (overdue, _, _) = repo
            .find_overdue(None, None)
            .await
            .expect("find overdue tasks");
        assert_eq!(
            overdue
                .iter()
                .map(|task| task.id.as_str())
                .collect::<Vec<_>>(),
            vec![inbox_overdue.id.as_str()]
        );

        let urgent = tag_repo
            .create("urgent".to_string())
            .await
            .expect("create urgent tag");
        let release = tag_repo
            .create("release".to_string())
            .await
            .expect("create release tag");
        repo.add_tags(
            &inbox_overdue.id,
            vec![urgent.id.clone(), release.id.clone(), urgent.id.clone()],
        )
        .await
        .expect("add tags");
        let tagged = repo
            .with_subtasks(vec![repo
                .find_by_id(&inbox_overdue.id)
                .await
                .unwrap()
                .unwrap()])
            .await
            .expect("hydrate tagged task")
            .pop()
            .expect("tagged task");
        assert_eq!(
            tagged
                .tags
                .iter()
                .map(|tag| tag.name.as_str())
                .collect::<Vec<_>>(),
            vec!["release", "urgent"]
        );

        repo.remove_tags(&inbox_overdue.id, vec![urgent.id.clone()])
            .await
            .expect("remove tag");
        let retagged = repo
            .with_subtasks(vec![repo
                .find_by_id(&inbox_overdue.id)
                .await
                .unwrap()
                .unwrap()])
            .await
            .expect("hydrate retagged task")
            .pop()
            .expect("retagged task");
        assert_eq!(
            retagged
                .tags
                .iter()
                .map(|tag| tag.name.as_str())
                .collect::<Vec<_>>(),
            vec!["release"]
        );

        cleanup_db(db_path);
    }
}
