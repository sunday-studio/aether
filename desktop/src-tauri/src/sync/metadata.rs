//! Sync metadata in _sync_meta: device_id, server_url, pull_cursor, key_salt, key_check, _suppress_triggers.

use crate::error::{AppError, Result};
use crate::sync::encryption;
use crate::sync::types::PullCursor;
use libsql::Database;
use sha2::{Digest, Sha256};

const KEY_DEVICE_ID: &str = "device_id";
const KEY_DEVICE_HOSTNAME: &str = "device_hostname";
const KEY_SERVER_URL: &str = "server_url";
const KEY_PULL_CURSOR: &str = "pull_cursor";
const KEY_LAST_SYNC: &str = "last_sync";
const KEY_SALT: &str = "key_salt";
const KEY_CHECK: &str = "key_check";
const KEY_DEVICE_TOKEN: &str = "device_token";
const KEY_SUPPRESS_TRIGGERS: &str = "_suppress_triggers";

pub async fn get_device_id(db: &Database) -> Result<String> {
    let v = get(db, KEY_DEVICE_ID).await?;
    match v {
        Some(s) => Ok(s),
        None => {
            let id = generate_device_id();
            set(db, KEY_DEVICE_ID, &id).await?;
            Ok(id)
        }
    }
}

fn generate_device_id() -> String {
    let machine_id = get_machine_id();
    let mut hasher = Sha256::new();
    hasher.update(b"aether-sync-device-v1:");
    hasher.update(machine_id.as_bytes());
    let hash = hasher.finalize();

    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]),
        u16::from_be_bytes([hash[4], hash[5]]),
        u16::from_be_bytes([hash[6], hash[7]]),
        u16::from_be_bytes([hash[8], hash[9]]),
        u64::from_be_bytes([0, 0, hash[10], hash[11], hash[12], hash[13], hash[14], hash[15]])
            & 0xffffffffffff
    )
}

fn get_machine_id() -> String {
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("ioreg")
            .args(["-rd1", "-c", "IOPlatformExpertDevice"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("IOPlatformUUID") {
                    if let Some(uuid) = line.split('"').nth(3) {
                        return uuid.to_string();
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
            return id.trim().to_string();
        }
        if let Ok(id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
            return id.trim().to_string();
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("reg")
            .args([
                "query",
                "HKLM\\SOFTWARE\\Microsoft\\Cryptography",
                "/v",
                "MachineGuid",
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("MachineGuid") {
                    if let Some(guid) = line.split_whitespace().last() {
                        return guid.to_string();
                    }
                }
            }
        }
    }

    gethostname::gethostname().to_string_lossy().to_string()
}

pub async fn get_device_hostname(db: &Database) -> Result<String> {
    let v = get(db, KEY_DEVICE_HOSTNAME).await?;
    match v {
        Some(s) => Ok(s),
        None => {
            let hostname = gethostname::gethostname().to_string_lossy().to_string();
            set(db, KEY_DEVICE_HOSTNAME, &hostname).await?;
            Ok(hostname)
        }
    }
}

pub async fn get_server_url(db: &Database) -> Result<Option<String>> {
    get(db, KEY_SERVER_URL).await
}

pub async fn set_server_url(db: &Database, url: &str) -> Result<()> {
    set(db, KEY_SERVER_URL, url).await
}

pub async fn get_device_token(db: &Database) -> Result<Option<String>> {
    get(db, KEY_DEVICE_TOKEN).await
}

pub async fn set_device_token(db: &Database, token: &str) -> Result<()> {
    set(db, KEY_DEVICE_TOKEN, token).await
}

pub async fn get_pull_cursor(db: &Database) -> Result<Option<PullCursor>> {
    let raw = get(db, KEY_PULL_CURSOR).await?;
    raw.and_then(|value| serde_json::from_str(&value).ok())
        .map(Ok)
        .unwrap_or(Ok(None))
}

pub async fn set_pull_cursor(db: &Database, cursor: &PullCursor) -> Result<()> {
    let raw = serde_json::to_string(cursor).map_err(AppError::Serialization)?;
    set(db, KEY_PULL_CURSOR, &raw).await?;
    set(db, KEY_LAST_SYNC, &cursor.received_at.to_string()).await
}

pub async fn clear_pull_cursor(db: &Database) -> Result<()> {
    delete_keys(db, &[KEY_PULL_CURSOR, KEY_LAST_SYNC]).await
}

pub async fn get_last_sync(db: &Database) -> Result<Option<i64>> {
    let v = get(db, KEY_LAST_SYNC).await?;
    Ok(v.and_then(|s| s.parse().ok()))
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
    set(db, KEY_SUPPRESS_TRIGGERS, value).await
}

pub async fn get_suppress_triggers(db: &Database) -> Result<String> {
    Ok(get(db, KEY_SUPPRESS_TRIGGERS)
        .await?
        .unwrap_or_else(|| "0".to_string()))
}

pub fn decode_server_salt(salt_b64: &str) -> Result<[u8; 16]> {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

    let salt_bytes = BASE64
        .decode(salt_b64)
        .map_err(|e| AppError::Sync(format!("invalid base64 salt: {}", e)))?;
    salt_bytes
        .try_into()
        .map_err(|_| AppError::Sync("invalid salt length from server".into()))
}

pub async fn configure_key(db: &Database, passphrase: &str, salt: &[u8; 16]) -> Result<[u8; 32]> {
    let key = encryption::derive_key(passphrase, salt)?;
    set_key_salt(db, salt).await?;
    set_key_check(db, &encryption::key_check_hash(&key)).await?;
    Ok(key)
}

pub async fn clear_key_material(db: &Database) -> Result<()> {
    delete_keys(db, &[KEY_SALT, KEY_CHECK]).await
}

pub async fn clear_sync_configuration(db: &Database) -> Result<()> {
    delete_keys(
        db,
        &[
            KEY_SERVER_URL,
            KEY_DEVICE_TOKEN,
            KEY_PULL_CURSOR,
            KEY_LAST_SYNC,
            KEY_SALT,
            KEY_CHECK,
        ],
    )
    .await
}

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
    if encryption::key_check_hash(&key) != stored {
        return Err(AppError::Sync("passphrase verification failed".into()));
    }
    Ok(key)
}

async fn get(db: &Database, key: &str) -> Result<Option<String>> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    let mut rows = conn
        .query(
            "SELECT value FROM _sync_meta WHERE key = ?1",
            libsql::params![key],
        )
        .await
        .map_err(AppError::LibSQL)?;
    if let Some(row) = rows.next().await.map_err(AppError::LibSQL)? {
        Ok(Some(row.get(0).map_err(AppError::LibSQL)?))
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

async fn delete_keys(db: &Database, keys: &[&str]) -> Result<()> {
    let conn = db.connect().map_err(AppError::LibSQL)?;
    for key in keys {
        conn.execute(
            "DELETE FROM _sync_meta WHERE key = ?1",
            libsql::params![key],
        )
        .await
        .map_err(AppError::LibSQL)?;
    }
    Ok(())
}
