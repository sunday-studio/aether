//! Sync metadata in _sync_meta: device_id, server_url, last_sync, key_salt, key_check, _suppress_triggers.

use crate::error::{AppError, Result};
use crate::sync::encryption;
use libsql::Database;
use uuid::Uuid;

const KEY_DEVICE_ID: &str = "device_id";
const KEY_SERVER_URL: &str = "server_url";
const KEY_LAST_SYNC: &str = "last_sync";
const KEY_SALT: &str = "key_salt";
const KEY_CHECK: &str = "key_check";
const KEY_SUPPRESS_TRIGGERS: &str = "_suppress_triggers";

pub async fn get_device_id(db: &Database) -> Result<String> {
    let v = get(db, KEY_DEVICE_ID).await?;
    match v {
        Some(s) => Ok(s),
        None => {
            let id = Uuid::new_v4().to_string();
            set(db, KEY_DEVICE_ID, &id).await?;
            Ok(id)
        }
    }
}

pub async fn get_server_url(db: &Database) -> Result<Option<String>> {
    get(db, KEY_SERVER_URL).await
}

pub async fn set_server_url(db: &Database, url: &str) -> Result<()> {
    set(db, KEY_SERVER_URL, url).await
}

pub async fn get_last_sync(db: &Database) -> Result<Option<i64>> {
    let v = get(db, KEY_LAST_SYNC).await?;
    Ok(v.and_then(|s| s.parse().ok()))
}

pub async fn set_last_sync(db: &Database, ts: i64) -> Result<()> {
    set(db, KEY_LAST_SYNC, &ts.to_string()).await
}

pub async fn get_key_salt(db: &Database) -> Result<Option<Vec<u8>>> {
    let v = get(db, KEY_SALT).await?;
    Ok(v.and_then(|s| hex::decode(s).ok()))
}

pub async fn set_key_salt(db: &Database, salt: &[u8]) -> Result<()> {
    set(db, KEY_SALT, &hex::encode(salt)).await
}

pub async fn get_key_check(db: &Database) -> Result<Option<String>> {
    get(db, KEY_CHECK).await
}

pub async fn set_key_check(db: &Database, hash: &str) -> Result<()> {
    set(db, KEY_CHECK, hash).await
}

pub async fn set_suppress_triggers(db: &Database, value: &str) -> Result<()> {
    tracing::debug!("[SYNC-META] Setting _suppress_triggers to '{}'", value);
    set(db, KEY_SUPPRESS_TRIGGERS, value).await
}

pub async fn get_suppress_triggers(db: &Database) -> Result<String> {
    let v = get(db, KEY_SUPPRESS_TRIGGERS).await?;
    Ok(v.unwrap_or_else(|| "0".to_string()))
}

/// Configure key material: generate salt, derive key, store salt and key_check.
/// Returns the derived key for immediate use; key is not stored.
pub async fn configure_key(db: &Database, passphrase: &str) -> Result<[u8; 32]> {
    let salt = encryption::generate_salt();
    let key = encryption::derive_key(passphrase, &salt)?;
    set_key_salt(db, &salt).await?;
    set_key_check(db, &encryption::key_check_hash(&key)).await?;
    Ok(key)
}

/// Verify passphrase: derive key from passphrase and stored salt, compare to key_check.
pub async fn verify_key(db: &Database, passphrase: &str) -> Result<[u8; 32]> {
    let salt = get_key_salt(db)
        .await?
        .ok_or_else(|| AppError::Sync("key_salt not set".into()))?;
    let salt_arr: [u8; 16] = salt
        .try_into()
        .map_err(|_| AppError::Sync("invalid key_salt length".into()))?;
    let key = encryption::derive_key(passphrase, &salt_arr)?;
    let stored = get_key_check(db)
        .await?
        .ok_or_else(|| AppError::Sync("key_check not set".into()))?;
    let computed = encryption::key_check_hash(&key);
    if computed != stored {
        return Err(AppError::Sync("passphrase verification failed".into()));
    }
    Ok(key)
}

async fn get(db: &Database, key: &str) -> Result<Option<String>> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query("SELECT value FROM _sync_meta WHERE key = ?1", libsql::params![key])
        .await
        .map_err(AppError::LibSQL)?;
    if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
        let v: String = row.get(0).map_err(AppError::LibSQL)?;
        Ok(Some(v))
    } else {
        Ok(None)
    }
}

async fn set(db: &Database, key: &str, value: &str) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    conn.execute(
        "INSERT OR REPLACE INTO _sync_meta (key, value) VALUES (?1, ?2)",
        libsql::params![key, value],
    )
    .await
    .map_err(AppError::LibSQL)?;
    Ok(())
}
