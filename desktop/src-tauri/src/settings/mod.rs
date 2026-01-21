pub mod encryption;

use crate::db::repositories::SettingsRepository;
use crate::error::Result;
use libsql::Database;
use std::sync::Arc;

/// Check if a setting key should be encrypted
fn is_sensitive_key(key: &str) -> bool {
    let key_lower = key.to_lowercase();
    key_lower.contains("api_key")
        || key_lower.contains("auth_token")
        || key_lower.contains("password")
        || key_lower.contains("secret")
}

/// Get a setting value (auto-decrypts if encrypted)
pub async fn get_setting(
    database: Arc<Database>,
    key: &str,
) -> Result<Option<String>> {
    let repo = SettingsRepository::new(database);
    let setting = repo.get(key).await?;
    
    if let Some(setting) = setting {
        // Check if value is encrypted (starts with encryption marker)
        if setting.value.starts_with("encrypted:") {
            let encrypted_value = setting.value.strip_prefix("encrypted:").unwrap();
            let decrypted = encryption::decrypt(encrypted_value)?;
            Ok(Some(decrypted))
        } else {
            Ok(Some(setting.value))
        }
    } else {
        Ok(None)
    }
}

/// Set a setting value (auto-encrypts if sensitive)
pub async fn set_setting(
    database: Arc<Database>,
    key: &str,
    value: &str,
) -> Result<()> {
    let repo = SettingsRepository::new(database);
    
    let value_to_store = if is_sensitive_key(key) {
        // Encrypt sensitive values
        let encrypted = encryption::encrypt(value)?;
        format!("encrypted:{}", encrypted)
    } else {
        value.to_string()
    };
    
    repo.set(key, &value_to_store).await?;
    Ok(())
}

/// Delete a setting
pub async fn delete_setting(
    database: Arc<Database>,
    key: &str,
) -> Result<()> {
    let repo = SettingsRepository::new(database);
    repo.delete(key).await?;
    Ok(())
}

/// Get all settings with a given prefix
pub async fn get_settings_by_prefix(
    database: Arc<Database>,
    prefix: &str,
) -> Result<Vec<(String, String)>> {
    let repo = SettingsRepository::new(database);
    let settings = repo.get_by_prefix(prefix).await?;
    
    let mut result = Vec::new();
    for setting in settings {
        let value = if setting.value.starts_with("encrypted:") {
            let encrypted_value = setting.value.strip_prefix("encrypted:").unwrap();
            encryption::decrypt(encrypted_value)?
        } else {
            setting.value
        };
        result.push((setting.key, value));
    }
    
    Ok(result)
}
