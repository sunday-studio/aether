use crate::db::models::{Goal, GoalInstance};
use crate::error::{AppError, Result};
use crate::utils::goal_period::{calculate_goal_period, should_create_new_goal_instance, RecurringGoal};
use crate::utils::{generate_id};
use crate::utils::timezone::get_goal_location;
use chrono::Utc;
use libsql::Database;
use std::sync::Arc;

pub struct GoalRepository {
    database: Arc<Database>,
}

impl GoalRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Get all goals
    pub async fn find_all(&self) -> Result<Vec<Goal>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT id, name, description, is_non_recurring, recurrence_type, recurrence_interval, recurrence_anchor, recurrence_meta, timezone, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM goals 
                 WHERE deleted_at IS NULL 
                 ORDER BY created_at DESC
                 LIMIT 1000",
                libsql::params![],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut goals = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            goals.push(self.row_to_goal(row)?);
        }

        Ok(goals)
    }

    /// Get goal by ID
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Goal>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT id, name, description, is_non_recurring, recurrence_type, recurrence_interval, recurrence_anchor, recurrence_meta, timezone, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM goals 
                 WHERE id = ?1 AND deleted_at IS NULL",
                libsql::params![id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        if let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            Ok(Some(self.row_to_goal(row)?))
        } else {
            Ok(None)
        }
    }

    /// Create a new goal with its first instance
    pub async fn create(
        &self,
        name: String,
        description: Option<String>,
        is_non_recurring: bool,
        recurrence_type: Option<String>,
        recurrence_interval: Option<i32>,
        recurrence_anchor: Option<chrono::DateTime<Utc>>,
        recurrence_meta: Option<serde_json::Value>,
        timezone: String,
    ) -> Result<Goal> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        conn.execute("BEGIN TRANSACTION", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let id = generate_id("goal");
        let now = Utc::now();
        let now_ms = now.timestamp_millis();
        let created_at_str = now.to_rfc3339();
        let updated_at_str = now.to_rfc3339();
        let recurrence_anchor_str = recurrence_anchor.map(|d| d.to_rfc3339());
        let recurrence_meta_str = recurrence_meta.as_ref().map(|m| serde_json::to_string(m).unwrap_or_default());

        // Insert goal
        conn.execute(
            "INSERT INTO goals (id, name, description, is_non_recurring, recurrence_type, recurrence_interval, recurrence_anchor, recurrence_meta, timezone, created_at, updated_at, _sync_id, _updated_at, _deleted, _extra) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 0, '{}')",
            libsql::params![
                id.clone(),
                name.clone(),
                description.as_ref().map(|s| s.as_str()),
                if is_non_recurring { 1 } else { 0 },
                recurrence_type.as_ref().map(|s| s.as_str()),
                recurrence_interval,
                recurrence_anchor_str.as_ref().map(|s| s.as_str()),
                recurrence_meta_str.as_ref().map(|s| s.as_str()),
                timezone.clone(),
                created_at_str,
                updated_at_str,
                id.clone(),
                now_ms,
            ],
        )
        .await
        .map_err(|e| {
            let _ = conn.execute("ROLLBACK", libsql::params![]);
            AppError::LibSQL(e)
        })?;

        // Create first goal instance
        let instance_id = generate_id("goal_instance");
        let instance_created_at_str = now.to_rfc3339();

        if is_non_recurring {
            // For non-recurring goals: create instance with periodStart=now, periodEnd=nil
            let instance_created_at_str_1 = instance_created_at_str.clone();
            let instance_created_at_str_2 = instance_created_at_str.clone();
            conn.execute(
                "INSERT INTO goal_instances (id, goal_id, period_start, period_end, status, created_at, updated_at, _sync_id, _updated_at, _deleted, _extra) 
                 VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, ?8, 0, '{}')",
                libsql::params![
                    instance_id.clone(),
                    id.clone(),
                    instance_created_at_str,
                    "active",
                    instance_created_at_str_1,
                    instance_created_at_str_2,
                    instance_id,
                    now_ms,
                ],
            )
            .await
            .map_err(|e| {
                let _ = conn.execute("ROLLBACK", libsql::params![]);
                AppError::LibSQL(e)
            })?;
        } else {
            // For recurring goals: calculate period using goal's timezone
            let tz = get_goal_location(&timezone)
                .map_err(|e| AppError::BadRequest(format!("Invalid timezone: {}", e)))?;

            let recurring_goal = RecurringGoal {
                recurrence_type: recurrence_type.clone().unwrap_or_default(),
                recurrence_interval: recurrence_interval.unwrap_or(1),
                recurrence_anchor: recurrence_anchor.unwrap_or(now),
            };

            let (period_start, period_end) = calculate_goal_period(recurring_goal, now, tz);
            let period_start_str = period_start.to_rfc3339();
            let period_end_str = period_end.to_rfc3339();
            let instance_created_at_str_1 = instance_created_at_str.clone();
            let instance_created_at_str_2 = instance_created_at_str.clone();

            conn.execute(
                "INSERT INTO goal_instances (id, goal_id, period_start, period_end, status, created_at, updated_at, _sync_id, _updated_at, _deleted, _extra) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, '{}')",
                libsql::params![
                    instance_id.clone(),
                    id.clone(),
                    period_start_str,
                    period_end_str,
                    "active",
                    instance_created_at_str_1,
                    instance_created_at_str_2,
                    instance_id,
                    now_ms,
                ],
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

        // Return created goal
        self.find_by_id(&id).await?.ok_or_else(|| AppError::Internal("Failed to retrieve created goal".to_string()))
    }

    /// Update a goal
    pub async fn update(
        &self,
        id: &str,
        name: Option<String>,
        description: Option<Option<String>>,
        is_non_recurring: Option<bool>,
        recurrence_type: Option<Option<String>>,
        recurrence_interval: Option<Option<i32>>,
        recurrence_anchor: Option<Option<chrono::DateTime<Utc>>>,
        recurrence_meta: Option<Option<serde_json::Value>>,
        client_updated_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<Goal> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Get current goal
        let current = self.find_by_id(id).await?;
        let mut goal = current.ok_or_else(|| AppError::NotFound(format!("Goal {} not found", id)))?;

        // Last-Write-Wins conflict detection
        if let Some(client_time) = client_updated_at {
            if client_time < goal.updated_at {
                return Err(AppError::BadRequest(format!(
                    "Conflict: Goal was modified by another device. Current updated_at: {}",
                    goal.updated_at.to_rfc3339()
                )));
            }
        }

        // Prevent changing IsNonRecurring field
        if let Some(is_non_recurring_val) = is_non_recurring {
            if is_non_recurring_val != goal.is_non_recurring {
                return Err(AppError::BadRequest(
                    "cannot change isNonRecurring field - goal type cannot be converted between recurring and non-recurring".to_string(),
                ));
            }
        }

        // Update fields
        if let Some(n) = name {
            goal.name = n;
        }
        if let Some(d) = description {
            goal.description = d;
        }

        // Update recurrence fields based on whether goal is non-recurring
        if goal.is_non_recurring {
            // For non-recurring goals, recurrence fields should remain nil
            if recurrence_type.is_some() || recurrence_interval.is_some() || recurrence_anchor.is_some() {
                return Err(AppError::BadRequest(
                    "cannot set recurrence fields for non-recurring goals".to_string(),
                ));
            }
        } else {
            // For recurring goals, allow updating recurrence fields
            if let Some(rt) = recurrence_type {
                goal.recurrence_type = rt;
            }
            if let Some(ri) = recurrence_interval {
                goal.recurrence_interval = ri;
            }
            if let Some(ra) = recurrence_anchor {
                goal.recurrence_anchor = ra;
            }
        }

        if let Some(rm) = recurrence_meta {
            goal.recurrence_meta = rm;
        }

        goal.updated_at = Utc::now();
        let now_ms = goal.updated_at.timestamp_millis();
        goal._updated_at = Some(now_ms);

        let updated_at_str = goal.updated_at.to_rfc3339();
        let recurrence_anchor_str = goal.recurrence_anchor.map(|d| d.to_rfc3339());
        let recurrence_meta_str = goal.recurrence_meta.as_ref().map(|m| serde_json::to_string(m).unwrap_or_default());

        conn.execute(
            "UPDATE goals 
             SET name = ?1, description = ?2, is_non_recurring = ?3, recurrence_type = ?4, recurrence_interval = ?5, recurrence_anchor = ?6, recurrence_meta = ?7, updated_at = ?8, _updated_at = ?9 
             WHERE id = ?10",
            libsql::params![
                goal.name.clone(),
                goal.description.as_ref().map(|s| s.as_str()),
                if goal.is_non_recurring { 1 } else { 0 },
                goal.recurrence_type.as_ref().map(|s| s.as_str()),
                goal.recurrence_interval,
                recurrence_anchor_str.as_ref().map(|s| s.as_str()),
                recurrence_meta_str.as_ref().map(|s| s.as_str()),
                updated_at_str,
                now_ms,
                id
            ],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(goal)
    }

    /// Delete a goal (soft delete)
    pub async fn delete(&self, id: &str) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Check if goal exists
        let goal = self.find_by_id(id).await?;
        if goal.is_none() {
            return Err(AppError::NotFound(format!("Goal {} not found", id)));
        }

        let now = Utc::now();
        let updated_at_str = now.to_rfc3339();
        let deleted_at_str = now.to_rfc3339();
        let now_ms = now.timestamp_millis();

        conn.execute("BEGIN TRANSACTION", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        // Soft delete goal
        let deleted_at_str_1 = deleted_at_str.clone();
        let updated_at_str_1 = updated_at_str.clone();
        conn.execute(
            "UPDATE goals SET deleted_at = ?1, updated_at = ?2, _updated_at = ?3, _deleted = 1 WHERE id = ?4",
            libsql::params![deleted_at_str_1, updated_at_str_1, now_ms, id],
        )
        .await
        .map_err(|e| {
            let _ = conn.execute("ROLLBACK", libsql::params![]);
            AppError::LibSQL(e)
        })?;

        // Soft delete instances
        conn.execute(
            "UPDATE goal_instances SET deleted_at = ?1, updated_at = ?2, _updated_at = ?3, _deleted = 1 WHERE goal_id = ?4",
            libsql::params![deleted_at_str, updated_at_str, now_ms, id],
        )
        .await
        .map_err(|e| {
            let _ = conn.execute("ROLLBACK", libsql::params![]);
            AppError::LibSQL(e)
        })?;

        conn.execute("COMMIT", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Get goal instances for a goal
    pub async fn find_instances(&self, goal_id: &str) -> Result<Vec<GoalInstance>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Verify goal exists
        let goal = self.find_by_id(goal_id).await?;
        if goal.is_none() {
            return Err(AppError::NotFound(format!("Goal {} not found", goal_id)));
        }

        let mut rows = conn
            .query(
                "SELECT id, goal_id, period_start, period_end, status, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM goal_instances 
                 WHERE goal_id = ?1 
                 ORDER BY period_start DESC",
                libsql::params![goal_id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut instances = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            instances.push(self.row_to_goal_instance(row)?);
        }

        Ok(instances)
    }

    /// Get or create current goal instance
    pub async fn get_or_create_current_instance(&self, goal_id: &str) -> Result<GoalInstance> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        conn.execute("BEGIN TRANSACTION", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        // Get goal
        let goal = self.find_by_id(goal_id).await?;
        let goal = goal.ok_or_else(|| AppError::NotFound(format!("Goal {} not found", goal_id)))?;

        // Get last instance
        let mut rows = conn
            .query(
                "SELECT id, goal_id, period_start, period_end, status, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                 FROM goal_instances 
                 WHERE goal_id = ?1 
                 ORDER BY created_at DESC 
                 LIMIT 1",
                libsql::params![goal_id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let last_instance = if let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            Some(self.row_to_goal_instance(row)?)
        } else {
            None
        };

        // For non-recurring goals, always return the same instance (or create if none exists)
        if goal.is_non_recurring {
            if let Some(instance) = last_instance {
                conn.execute("COMMIT", libsql::params![])
                    .await
                    .map_err(|e| AppError::LibSQL(e))?;
                return Ok(instance);
            }

            // Create instance for non-recurring goal
            let instance_id = generate_id("goal_instance");
            let now = Utc::now();
            let now_ms = now.timestamp_millis();
            let created_at_str = now.to_rfc3339();
            let created_at_str_1 = created_at_str.clone();
            let updated_at_str = created_at_str.clone();

            conn.execute(
                "INSERT INTO goal_instances (id, goal_id, period_start, period_end, status, created_at, updated_at, _sync_id, _updated_at, _deleted, _extra) 
                 VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, ?8, 0, '{}')",
                libsql::params![
                    instance_id.clone(),
                    goal_id,
                    created_at_str,
                    "active",
                    created_at_str_1,
                    updated_at_str,
                    instance_id.clone(),
                    now_ms,
                ],
            )
            .await
            .map_err(|e| {
                let _ = conn.execute("ROLLBACK", libsql::params![]);
                AppError::LibSQL(e)
            })?;

            conn.execute("COMMIT", libsql::params![])
                .await
                .map_err(|e| AppError::LibSQL(e))?;

            // Return created instance
            let mut rows = conn
                .query(
                    "SELECT id, goal_id, period_start, period_end, status, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                     FROM goal_instances 
                     WHERE id = ?1",
                    libsql::params![instance_id],
                )
                .await
                .map_err(|e| AppError::LibSQL(e))?;

            if let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
                return Ok(self.row_to_goal_instance(row)?);
            } else {
                return Err(AppError::Internal("Failed to retrieve created instance".to_string()));
            }
        }

        // For recurring goals, check if new instance should be created
        let should_create = should_create_new_goal_instance(
            goal.is_non_recurring,
            last_instance.as_ref().map(|i| i.created_at),
            goal.recurrence_interval,
            Some(&goal.timezone),
        )?;

        if should_create {
            // Get goal's timezone location
            let tz = get_goal_location(&goal.timezone)
                .map_err(|e| AppError::BadRequest(format!("Invalid timezone: {}", e)))?;

            // Calculate new period
            let recurring_goal = RecurringGoal {
                recurrence_type: goal.recurrence_type.clone().unwrap_or_default(),
                recurrence_interval: goal.recurrence_interval.unwrap_or(1),
                recurrence_anchor: goal.recurrence_anchor.unwrap_or(Utc::now()),
            };

            let now = Utc::now();
            let now_ms = now.timestamp_millis();
            let (period_start, period_end) = calculate_goal_period(recurring_goal, now, tz);
            let period_start_str = period_start.to_rfc3339();
            let period_end_str = period_end.to_rfc3339();

            let instance_id = generate_id("goal_instance");
            let created_at_str = now.to_rfc3339();
            let updated_at_str = created_at_str.clone();

            conn.execute(
                "INSERT INTO goal_instances (id, goal_id, period_start, period_end, status, created_at, updated_at, _sync_id, _updated_at, _deleted, _extra) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, '{}')",
                libsql::params![
                    instance_id.clone(),
                    goal_id,
                    period_start_str,
                    period_end_str,
                    "active",
                    created_at_str,
                    updated_at_str,
                    instance_id.clone(),
                    now_ms,
                ],
            )
            .await
            .map_err(|e| {
                let _ = conn.execute("ROLLBACK", libsql::params![]);
                AppError::LibSQL(e)
            })?;

            conn.execute("COMMIT", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

            // Return created instance
            let mut rows = conn
                .query(
                    "SELECT id, goal_id, period_start, period_end, status, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra 
                     FROM goal_instances 
                     WHERE id = ?1",
                    libsql::params![instance_id],
                )
                .await
                .map_err(|e| AppError::LibSQL(e))?;

            if let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
                return Ok(self.row_to_goal_instance(row)?);
            } else {
                return Err(AppError::Internal("Failed to retrieve created instance".to_string()));
            }
        }

        // Return last instance (no new instance needed)
        conn.execute("COMMIT", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        if let Some(instance) = last_instance {
            Ok(instance)
        } else {
            Err(AppError::Internal("No instance found and failed to create one".to_string()))
        }
    }

    /// Add tags to a goal
    pub async fn add_tags(&self, goal_id: &str, tag_ids: Vec<String>) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Verify goal exists
        let goal = self.find_by_id(goal_id).await?;
        if goal.is_none() {
            return Err(AppError::NotFound(format!("Goal {} not found", goal_id)));
        }

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

        for tag_id in &tag_ids {
            let sync_id = format!("{}|{}", goal_id, tag_id);
            conn.execute(
                "INSERT OR IGNORE INTO goal_tags (goal_id, tag_id, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, 0, '{}')",
                libsql::params![goal_id, tag_id.as_str(), sync_id, now_ms],
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

    /// Remove tags from a goal
    pub async fn remove_tags(&self, goal_id: &str, tag_ids: Vec<String>) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        // Verify goal exists
        let goal = self.find_by_id(goal_id).await?;
        if goal.is_none() {
            return Err(AppError::NotFound(format!("Goal {} not found", goal_id)));
        }

        if tag_ids.is_empty() {
            return Ok(());
        }

        conn.execute("BEGIN TRANSACTION", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        for tag_id in &tag_ids {
            conn.execute(
                "DELETE FROM goal_tags WHERE goal_id = ?1 AND tag_id = ?2",
                libsql::params![goal_id, tag_id.as_str()],
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

    /// Helper to convert database row to Goal
    fn row_to_goal(&self, row: libsql::Row) -> Result<Goal> {
        let id: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
        let name: String = row.get(1).map_err(|e| AppError::LibSQL(e))?;
        let description: Option<String> = row.get(2).map_err(|e| AppError::LibSQL(e))?;
        let is_non_recurring: i64 = row.get(3).map_err(|e| AppError::LibSQL(e))?;
        let recurrence_type: Option<String> = row.get(4).map_err(|e| AppError::LibSQL(e))?;
        let recurrence_interval: Option<i64> = row.get(5).map_err(|e| AppError::LibSQL(e))?;
        let recurrence_anchor_str: Option<String> = row.get(6).map_err(|e| AppError::LibSQL(e))?;
        let recurrence_meta_str: Option<String> = row.get(7).map_err(|e| AppError::LibSQL(e))?;
        let timezone: String = row.get(8).map_err(|e| AppError::LibSQL(e))?;
        let created_at_str: String = row.get(9).map_err(|e| AppError::LibSQL(e))?;
        let updated_at_str: String = row.get(10).map_err(|e| AppError::LibSQL(e))?;
        let deleted_at_str: Option<String> = row.get(11).map_err(|e| AppError::LibSQL(e))?;

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid updated_at: {}", e)))?
            .with_timezone(&Utc);
        let recurrence_anchor = recurrence_anchor_str
            .map(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .flatten()
            .map(|dt| dt.with_timezone(&Utc));
        let deleted_at = deleted_at_str
            .map(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .flatten()
            .map(|dt| dt.with_timezone(&Utc));
        let recurrence_meta = recurrence_meta_str
            .map(|s| serde_json::from_str(&s).ok())
            .flatten();

        let _sync_id: Option<String> = row.get(12).ok();
        let _updated_at: Option<i64> = row.get(13).ok();
        let _deleted: i64 = row.get(14).unwrap_or(0);
        let _extra: Option<serde_json::Value> = row.get::<Option<String>>(15).ok().flatten().and_then(|s| serde_json::from_str(&s).ok());

        Ok(Goal {
            id,
            name,
            description,
            is_non_recurring: is_non_recurring != 0,
            recurrence_type,
            recurrence_interval: recurrence_interval.map(|i| i as i32),
            recurrence_anchor,
            recurrence_meta,
            timezone,
            created_at,
            updated_at,
            deleted_at,
            _sync_id,
            _updated_at,
            _deleted: _deleted != 0,
            _extra,
        })
    }

    /// Helper to convert database row to GoalInstance
    fn row_to_goal_instance(&self, row: libsql::Row) -> Result<GoalInstance> {
        let id: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
        let goal_id: String = row.get(1).map_err(|e| AppError::LibSQL(e))?;
        let period_start_str: String = row.get(2).map_err(|e| AppError::LibSQL(e))?;
        let period_end_str: Option<String> = row.get(3).map_err(|e| AppError::LibSQL(e))?;
        let status: String = row.get(4).map_err(|e| AppError::LibSQL(e))?;
        let created_at_str: String = row.get(5).map_err(|e| AppError::LibSQL(e))?;
        let _sync_id: Option<String> = row.get(8).ok();
        let _updated_at: Option<i64> = row.get(9).ok();
        let _deleted: i64 = row.get(10).unwrap_or(0);
        let _extra: Option<serde_json::Value> = row.get::<Option<String>>(11).ok().flatten().and_then(|s| serde_json::from_str(&s).ok());

        let period_start = chrono::DateTime::parse_from_rfc3339(&period_start_str)
            .map_err(|e| AppError::Internal(format!("Invalid period_start: {}", e)))?
            .with_timezone(&Utc);
        let period_end = period_end_str
            .map(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .flatten()
            .map(|dt| dt.with_timezone(&Utc));
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&Utc);

        Ok(GoalInstance {
            id,
            goal_id,
            period_start,
            period_end,
            status,
            created_at,
            _sync_id,
            _updated_at,
            _deleted: _deleted != 0,
            _extra,
        })
    }
}
