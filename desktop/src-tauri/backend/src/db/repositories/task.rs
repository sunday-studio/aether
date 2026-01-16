use crate::db::models::{SubTask, Task};
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

        conn.execute(
            "INSERT INTO tasks (id, title, description, is_completed, due_date, goal_id, goal_instance_id, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            libsql::params![
                id.clone(),
                title.clone(),
                description.as_ref().map(|s| s.as_str()),
                if false { 1 } else { 0 }, // is_completed
                due_date_str.as_ref().map(|s| s.as_str()),
                goal_id.as_ref().map(|s| s.as_str()),
                goal_instance_id.as_ref().map(|s| s.as_str()),
                created_at_str,
                updated_at_str
            ],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(Task {
            id,
            title,
            description,
            is_completed: false,
            due_date,
            goal_instance_id,
            goal_id,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }

    /// Get inbox tasks (tasks not attached to any goal)
    pub async fn find_inbox(&self) -> Result<Vec<Task>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT id, title, description, is_completed, due_date, goal_instance_id, goal_id, created_at, updated_at, deleted_at 
                 FROM tasks 
                 WHERE goal_id IS NULL AND deleted_at IS NULL 
                 ORDER BY due_date ASC",
                libsql::params![],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut tasks = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            tasks.push(self.row_to_task(row)?);
        }

        Ok(tasks)
    }

    /// Get overdue tasks
    pub async fn find_overdue(&self) -> Result<Vec<Task>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        let mut rows = conn
            .query(
                "SELECT id, title, description, is_completed, due_date, goal_instance_id, goal_id, created_at, updated_at, deleted_at 
                 FROM tasks 
                 WHERE due_date < ?1 AND is_completed = 0 AND deleted_at IS NULL 
                 ORDER BY due_date ASC",
                libsql::params![now_str],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut tasks = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            tasks.push(self.row_to_task(row)?);
        }

        Ok(tasks)
    }

    /// Get task by ID
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Task>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT id, title, description, is_completed, due_date, goal_instance_id, goal_id, created_at, updated_at, deleted_at 
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

        let updated_at_str = task.updated_at.to_rfc3339();
        let due_date_str = task.due_date.map(|d| d.to_rfc3339());

        conn.execute(
            "UPDATE tasks 
             SET title = ?1, description = ?2, is_completed = ?3, due_date = ?4, goal_id = ?5, goal_instance_id = ?6, updated_at = ?7 
             WHERE id = ?8",
            libsql::params![
                task.title.clone(),
                task.description.as_ref().map(|s| s.as_str()),
                if task.is_completed { 1 } else { 0 },
                due_date_str.as_ref().map(|s| s.as_str()),
                task.goal_id.as_ref().map(|s| s.as_str()),
                task.goal_instance_id.as_ref().map(|s| s.as_str()),
                updated_at_str,
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

        conn.execute(
            "UPDATE tasks SET deleted_at = ?1, updated_at = ?2 WHERE id = ?3",
            libsql::params![deleted_at_str, updated_at_str, id],
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
                "SELECT id, title, is_completed, task_id, order_index, created_at, updated_at, deleted_at 
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

        conn.execute(
            "INSERT INTO subtasks (id, title, is_completed, task_id, order_index, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            libsql::params![
                id.clone(),
                title.clone(),
                if false { 1 } else { 0 }, // is_completed
                task_id,
                order_index,
                created_at_str,
                updated_at_str
            ],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(SubTask {
            id,
            title,
            is_completed: false,
            task_id: task_id.to_string(),
            order_index,
            created_at: now,
            updated_at: now,
            deleted_at: None,
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
                "SELECT id, title, is_completed, task_id, order_index, created_at, updated_at, deleted_at 
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

        let updated_at_str = subtask.updated_at.to_rfc3339();

        conn.execute(
            "UPDATE subtasks SET title = ?1, is_completed = ?2, updated_at = ?3 WHERE id = ?4",
            libsql::params![
                subtask.title.clone(),
                if subtask.is_completed { 1 } else { 0 },
                updated_at_str,
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

        conn.execute(
            "UPDATE subtasks SET deleted_at = ?1, updated_at = ?2 WHERE id = ?3",
            libsql::params![deleted_at_str, updated_at_str, subtask_id],
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

        conn.execute("BEGIN TRANSACTION", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        for (index, subtask_id) in subtask_ids.iter().enumerate() {
            conn.execute(
                "UPDATE subtasks SET order_index = ?1 WHERE id = ?2 AND task_id = ?3",
                libsql::params![index as i32, subtask_id.as_str(), task_id],
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

        // Build placeholders for IN clause
        let placeholders: Vec<String> = (1..=tag_ids.len())
            .map(|i| format!("?{}", i))
            .collect();
        let query = format!(
            "SELECT id FROM tags WHERE id IN ({})",
            placeholders.join(", ")
        );

        // Check tags exist - we'll use a simpler approach
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

        for tag_id in tag_ids {
            // Use INSERT OR IGNORE to skip duplicates
            conn.execute(
                "INSERT OR IGNORE INTO task_tags (task_id, tag_id) VALUES (?1, ?2)",
                libsql::params![task_id, tag_id],
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

    /// Add goal to a task (will be implemented with goal instance logic later)
    pub async fn add_goal(&self, task_id: &str, goal_id: &str) -> Result<Option<String>> {
        // This will be implemented in Milestone 5 when we have goal instance logic
        // For now, return the goal_id as goal_instance_id placeholder
        Ok(Some(goal_id.to_string()))
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

        conn.execute(
            "UPDATE tasks SET goal_id = NULL, goal_instance_id = NULL, updated_at = ?1 WHERE id = ?2",
            libsql::params![updated_at_str, task_id],
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
        })
    }
}
