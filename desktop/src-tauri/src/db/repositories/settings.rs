use crate::db::models::Setting;
use crate::error::{AppError, Result};
use chrono::Utc;
use libsql::Database;
use std::sync::Arc;

pub struct SettingsRepository {
    database: Arc<Database>,
}

impl SettingsRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Get a setting by key
    pub async fn get(&self, key: &str) -> Result<Option<Setting>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT key, value, updated_at FROM settings WHERE key = ?1",
                libsql::params![key],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        if let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            Ok(Some(self.row_to_setting(row)?))
        } else {
            Ok(None)
        }
    }

    /// Set a setting (insert or update)
    pub async fn set(&self, key: &str, value: &str) -> Result<Setting> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let now = Utc::now();
        let updated_at_str = now.to_rfc3339();

        // Try INSERT ... ON CONFLICT first (works in local mode)
        // If it fails with "unsupported statement", fall back to check-then-insert/update
        let result = conn.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = ?3",
            libsql::params![key, value, updated_at_str.clone()],
        )
        .await;

        match result {
            Ok(_) => {
                // Success with UPSERT syntax
            }
            Err(e) => {
                let error_msg = e.to_string();
                // If UPSERT is not supported, use the fallback approach
                if error_msg.contains("unsupported") || error_msg.contains("ON CONFLICT") {
                    // Check if setting exists
                    let existing = self.get(key).await?;
                    
                    if existing.is_some() {
                        // Update existing setting
                        conn.execute(
                            "UPDATE settings SET value = ?1, updated_at = ?2 WHERE key = ?3",
                            libsql::params![value, updated_at_str.clone(), key],
                        )
                        .await
                        .map_err(|e| AppError::LibSQL(e))?;
                    } else {
                        // Insert new setting
                        conn.execute(
                            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
                            libsql::params![key, value, updated_at_str],
                        )
                        .await
                        .map_err(|e| AppError::LibSQL(e))?;
                    }
                } else {
                    // Some other error, propagate it
                    return Err(AppError::LibSQL(e));
                }
            }
        }

        Ok(Setting {
            key: key.to_string(),
            value: value.to_string(),
            updated_at: now,
        })
    }

    /// Delete a setting
    pub async fn delete(&self, key: &str) -> Result<()> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        conn.execute(
            "DELETE FROM settings WHERE key = ?1",
            libsql::params![key],
        )
        .await
        .map_err(|e| AppError::LibSQL(e))?;

        Ok(())
    }

    /// Get all settings
    pub async fn get_all(&self) -> Result<Vec<Setting>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let mut rows = conn
            .query(
                "SELECT key, value, updated_at FROM settings ORDER BY key ASC",
                libsql::params![],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut settings = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            settings.push(self.row_to_setting(row)?);
        }

        Ok(settings)
    }

    /// Get all settings with a given prefix
    pub async fn get_by_prefix(&self, prefix: &str) -> Result<Vec<Setting>> {
        let conn = self.database.connect().map_err(|e| AppError::LibSQL(e))?;
        
        let pattern = format!("{}%", prefix);
        let mut rows = conn
            .query(
                "SELECT key, value, updated_at FROM settings WHERE key LIKE ?1 ORDER BY key ASC",
                libsql::params![pattern],
            )
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        let mut settings = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AppError::LibSQL(e))? {
            settings.push(self.row_to_setting(row)?);
        }

        Ok(settings)
    }

    /// Helper to convert database row to Setting
    fn row_to_setting(&self, row: libsql::Row) -> Result<Setting> {
        let key: String = row.get(0).map_err(|e| AppError::LibSQL(e))?;
        let value: String = row.get(1).map_err(|e| AppError::LibSQL(e))?;
        let updated_at_str: String = row.get(2).map_err(|e| AppError::LibSQL(e))?;

        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| AppError::Internal(format!("Invalid updated_at: {}", e)))?
            .with_timezone(&Utc);

        Ok(Setting {
            key,
            value,
            updated_at,
        })
    }
}
