use crate::db::models::Activity;
use crate::error::{AppError, Result};
use crate::utils::generate_id;
use chrono::Utc;
use libsql::Database;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ActivityRepository {
    database: Arc<Database>,
}

impl ActivityRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Create a new activity record
    pub async fn create(
        &self,
        action_type: String,
        entity_type: String,
        entity_id: String,
        metadata: Option<serde_json::Value>,
    ) -> Result<Activity> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        let id = generate_id("activity");
        let created_at = Utc::now();
        let created_at_str = created_at.to_rfc3339();
        let metadata_str = metadata
            .as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default());

        conn.execute(
            "INSERT INTO activities (id, action_type, entity_type, entity_id, created_at, metadata) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            libsql::params![
                id.clone(),
                action_type.clone(),
                entity_type.clone(),
                entity_id.clone(),
                created_at_str,
                metadata_str
            ],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(Activity {
            id,
            action_type,
            entity_type,
            entity_id,
            created_at,
            metadata,
        })
    }

    /// Get activities grouped by date with detailed breakdowns
    /// Returns: date -> entity_type -> action_type -> count
    pub async fn get_by_date_range(
        &self,
        start_date: Option<chrono::DateTime<Utc>>,
        end_date: Option<chrono::DateTime<Utc>>,
    ) -> Result<HashMap<String, HashMap<String, HashMap<String, i64>>>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        // Default to last year if not specified
        let end = end_date.unwrap_or_else(Utc::now);
        let start = start_date.unwrap_or_else(|| end - chrono::Duration::days(365));

        let start_str = start.to_rfc3339();
        let end_str = end.to_rfc3339();

        // Use strftime to extract date from RFC3339 timestamp
        // SQLite's DATE() function works with ISO 8601 dates stored as TEXT
        let mut rows = conn
            .query(
                "SELECT action_type, entity_type, DATE(created_at) as activity_date, COUNT(*) as count
                 FROM activities
                 WHERE created_at >= ?1 AND created_at <= ?2
                 GROUP BY action_type, entity_type, DATE(created_at)
                 ORDER BY activity_date ASC",
                libsql::params![start_str, end_str],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut result: HashMap<String, HashMap<String, HashMap<String, i64>>> = HashMap::new();

        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            let action_type: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
            let entity_type: String = row.get(1).map_err(|e| AppError::LibSQL(e))?;
            let activity_date: String = row.get(2).map_err(|e| AppError::LibSQL(e))?;
            let count: i64 = row.get(3).map_err(|e| AppError::LibSQL(e))?;

            result
                .entry(activity_date)
                .or_insert_with(HashMap::new)
                .entry(entity_type)
                .or_insert_with(HashMap::new)
                .insert(action_type, count);
        }

        Ok(result)
    }

    /// Get audit log for a specific entity
    pub async fn get_by_entity(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<Activity>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        let mut rows = conn
            .query(
                "SELECT id, action_type, entity_type, entity_id, created_at, metadata
                 FROM activities
                 WHERE entity_type = ?1 AND entity_id = ?2
                 ORDER BY created_at DESC",
                libsql::params![entity_type, entity_id],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut activities = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            activities.push(self.row_to_activity(row)?);
        }

        Ok(activities)
    }

    /// Get all activities (for audit purposes)
    pub async fn get_all(&self, limit: Option<i64>) -> Result<Vec<Activity>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;

        let query = if let Some(limit_val) = limit {
            format!(
                "SELECT id, action_type, entity_type, entity_id, created_at, metadata
                 FROM activities
                 ORDER BY created_at DESC
                 LIMIT {}",
                limit_val
            )
        } else {
            "SELECT id, action_type, entity_type, entity_id, created_at, metadata
             FROM activities
             ORDER BY created_at DESC"
                .to_string()
        };

        let mut rows = conn
            .query(&query, libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut activities = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            activities.push(self.row_to_activity(row)?);
        }

        Ok(activities)
    }

    fn row_to_activity(&self, row: libsql::Row) -> Result<Activity> {
        let id: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
        let action_type: String = row.get(1).map_err(|e| AppError::LibSQL(e))?;
        let entity_type: String = row.get(2).map_err(|e| AppError::LibSQL(e))?;
        let entity_id: String = row.get(3).map_err(|e| AppError::LibSQL(e))?;
        let created_at_str: String = row.get(4).map_err(|e| AppError::LibSQL(e))?;
        let metadata_str: Option<String> = row.get(5).map_err(|e| AppError::LibSQL(e))?;

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| AppError::Internal(format!("Failed to parse created_at: {}", e)))?
            .with_timezone(&Utc);

        let metadata = if let Some(meta_str) = metadata_str {
            if meta_str.is_empty() {
                None
            } else {
                serde_json::from_str(&meta_str).ok()
            }
        } else {
            None
        };

        Ok(Activity {
            id,
            action_type,
            entity_type,
            entity_id,
            created_at,
            metadata,
        })
    }
}
