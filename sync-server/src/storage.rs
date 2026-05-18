//! SQLite storage for encrypted changes and blob metadata. Blob files in {data}/blobs/{hash}.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use crate::models::PullCursor;

/// Device row from the persistent devices table.
#[derive(Debug, Clone)]
pub struct DeviceRow {
    pub id: String,
    pub hostname: Option<String>,
    pub created_at: i64,
    pub last_seen: i64,
    pub last_sync: i64,
}

#[derive(Debug, Clone)]
pub struct StoredChange {
    pub id: i64,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub received_at: i64,
}

pub enum PushAcceptance {
    Accepted,
    Duplicate,
}

pub struct Storage {
    db_path: PathBuf,
    blob_dir: std::path::PathBuf,
}

impl Storage {
    pub fn new(db_path: &Path, data_root: &Path) -> Result<Self, rusqlite::Error> {
        std::fs::create_dir_all(db_path.parent().unwrap_or(Path::new("."))).ok();
        let db_path = db_path.to_path_buf();
        let conn = open_connection(&db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS changes (
                id INTEGER PRIMARY KEY,
                device_id TEXT NOT NULL,
                device_hostname TEXT,
                nonce BLOB NOT NULL,
                ciphertext BLOB NOT NULL,
                received_at INTEGER NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_changes_cursor ON changes(received_at, id)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS blobs (
                hash TEXT PRIMARY KEY,
                size INTEGER NOT NULL,
                uploaded_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS server_config (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS devices (
                id TEXT PRIMARY KEY NOT NULL,
                hostname TEXT,
                device_token_hash TEXT NOT NULL,
                token_created_at INTEGER NOT NULL,
                revoked_at INTEGER,
                created_at INTEGER NOT NULL,
                last_seen INTEGER NOT NULL,
                last_sync INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS processed_push_batches (
                device_id TEXT NOT NULL,
                batch_id TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (device_id, batch_id)
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_processed_push_batches_created_at ON processed_push_batches(created_at)",
            [],
        )?;

        let blob_dir = data_root.join("blobs");
        std::fs::create_dir_all(&blob_dir).ok();

        Ok(Self { db_path, blob_dir })
    }

    fn connect(&self) -> Result<Connection, rusqlite::Error> {
        open_connection(&self.db_path)
    }

    /// Initialize encryption salt on first startup. Generates a 16-byte random salt if none exists.
    pub fn initialize_salt(&self) -> Result<(), rusqlite::Error> {
        let conn = self.connect()?;

        let existing: Option<String> = conn
            .query_row(
                "SELECT value FROM server_config WHERE key = 'encryption_salt'",
                [],
                |r| r.get(0),
            )
            .optional()?;

        if existing.is_none() {
            let mut salt = [0u8; 16];
            rand::thread_rng().fill_bytes(&mut salt);
            let salt_b64 = BASE64.encode(salt);
            let now = epoch_secs();

            conn.execute(
                "INSERT INTO server_config (key, value, created_at) VALUES ('encryption_salt', ?1, ?2)",
                params![salt_b64, now],
            )?;

            tracing::info!("Generated new encryption salt (16 bytes)");
        } else {
            tracing::info!("Using existing encryption salt");
        }

        Ok(())
    }

    pub fn get_salt(&self) -> Result<String, rusqlite::Error> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT value FROM server_config WHERE key = 'encryption_salt'",
            [],
            |r| r.get(0),
        )
    }

    pub fn register_device(
        &self,
        device_id: &str,
        hostname: Option<&str>,
        device_token: &str,
    ) -> Result<(), rusqlite::Error> {
        let now = epoch_millis();
        let token_hash = hash_token(device_token);
        let conn = self.connect()?;
        let existing: Option<i64> = conn
            .query_row(
                "SELECT 1 FROM devices WHERE id = ?1",
                params![device_id],
                |_| Ok(1i64),
            )
            .optional()?;
        if existing.is_some() {
            conn.execute(
                "UPDATE devices
                 SET hostname = ?1, device_token_hash = ?2, token_created_at = ?3, revoked_at = NULL, last_seen = ?3, last_sync = ?3
                 WHERE id = ?4",
                params![hostname, token_hash, now, device_id],
            )?;
        } else {
            conn.execute(
                "INSERT INTO devices (id, hostname, device_token_hash, token_created_at, revoked_at, created_at, last_seen, last_sync)
                 VALUES (?1, ?2, ?3, ?4, NULL, ?4, ?4, ?4)",
                params![device_id, hostname, token_hash, now],
            )?;
        }
        Ok(())
    }

    pub fn authenticate_device(
        &self,
        device_id: &str,
        token: &str,
    ) -> Result<bool, rusqlite::Error> {
        let conn = self.connect()?;
        let stored_hash: Option<String> = conn
            .query_row(
                "SELECT device_token_hash FROM devices WHERE id = ?1 AND revoked_at IS NULL",
                params![device_id],
                |r| r.get(0),
            )
            .optional()?;
        Ok(stored_hash
            .map(|hash| hash == hash_token(token))
            .unwrap_or(false))
    }

    pub fn record_push_if_new(
        &self,
        device_id: &str,
        batch_id: &str,
        device_hostname: Option<&str>,
        changes: &[(Vec<u8>, Vec<u8>)],
    ) -> Result<PushAcceptance, rusqlite::Error> {
        let now = epoch_millis();
        let conn = self.connect()?;
        let tx = conn.unchecked_transaction()?;
        let inserted = tx.execute(
            "INSERT OR IGNORE INTO processed_push_batches (device_id, batch_id, created_at) VALUES (?1, ?2, ?3)",
            params![device_id, batch_id, now],
        )?;
        if inserted == 0 {
            tx.commit()?;
            return Ok(PushAcceptance::Duplicate);
        }

        for (nonce, ciphertext) in changes {
            tx.execute(
                "INSERT INTO changes (device_id, device_hostname, nonce, ciphertext, received_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![device_id, device_hostname, nonce, ciphertext, now],
            )?;
        }

        tx.execute(
            "DELETE FROM processed_push_batches WHERE created_at < ?1",
            params![now - 7 * 24 * 60 * 60 * 1000_i64],
        )?;
        tx.commit()?;
        Ok(PushAcceptance::Accepted)
    }

    pub fn pull(
        &self,
        cursor: Option<&PullCursor>,
        limit: i64,
    ) -> Result<Vec<StoredChange>, rusqlite::Error> {
        let conn = self.connect()?;
        let mut out = Vec::new();
        if let Some(cursor) = cursor {
            let mut stmt = conn.prepare(
                "SELECT id, nonce, ciphertext, received_at
                 FROM changes
                 WHERE (received_at > ?1) OR (received_at = ?1 AND id > ?2)
                 ORDER BY received_at, id
                 LIMIT ?3",
            )?;
            let rows =
                stmt.query_map(params![cursor.received_at, cursor.change_id, limit], |r| {
                    Ok(StoredChange {
                        id: r.get(0)?,
                        nonce: r.get(1)?,
                        ciphertext: r.get(2)?,
                        received_at: r.get(3)?,
                    })
                })?;
            for row in rows {
                out.push(row?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, nonce, ciphertext, received_at
                 FROM changes
                 ORDER BY received_at, id
                 LIMIT ?1",
            )?;
            let rows = stmt.query_map(params![limit], |r| {
                Ok(StoredChange {
                    id: r.get(0)?,
                    nonce: r.get(1)?,
                    ciphertext: r.get(2)?,
                    received_at: r.get(3)?,
                })
            })?;
            for row in rows {
                out.push(row?);
            }
        }
        Ok(out)
    }

    pub fn has_more_after(&self, cursor: &PullCursor) -> Result<bool, rusqlite::Error> {
        let conn = self.connect()?;
        let next: Option<i64> = conn
            .query_row(
                "SELECT id
                 FROM changes
                 WHERE (received_at > ?1) OR (received_at = ?1 AND id > ?2)
                 ORDER BY received_at, id
                 LIMIT 1",
                params![cursor.received_at, cursor.change_id],
                |r| r.get(0),
            )
            .optional()?;
        Ok(next.is_some())
    }

    pub fn put_blob(
        &self,
        hash: &str,
        data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let p = self.blob_path(hash)?;
        std::fs::write(&p, data)?;
        let conn = self.connect()?;
        let at = epoch_millis();
        conn.execute(
            "INSERT OR REPLACE INTO blobs (hash, size, uploaded_at) VALUES (?1, ?2, ?3)",
            params![hash, data.len() as i64, at],
        )?;
        Ok(())
    }

    pub fn get_blob(
        &self,
        hash: &str,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
        let p = self.blob_path(hash)?;
        if p.exists() {
            Ok(Some(std::fs::read(&p)?))
        } else {
            Ok(None)
        }
    }

    pub fn has_blob(&self, hash: &str) -> bool {
        self.blob_path(hash).is_ok_and(|path| path.exists())
    }

    fn blob_path(&self, hash: &str) -> Result<PathBuf, std::io::Error> {
        if valid_media_hash(hash) {
            Ok(self.blob_dir.join(hash))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid media hash",
            ))
        }
    }

    pub fn update_device_last_seen(&self, device_id: &str, ts: i64) -> Result<(), rusqlite::Error> {
        let conn = self.connect()?;
        conn.execute(
            "UPDATE devices SET last_seen = ?1 WHERE id = ?2",
            params![ts, device_id],
        )?;
        Ok(())
    }

    pub fn update_device_last_sync(&self, device_id: &str, ts: i64) -> Result<(), rusqlite::Error> {
        let conn = self.connect()?;
        conn.execute(
            "UPDATE devices SET last_sync = ?1 WHERE id = ?2",
            params![ts, device_id],
        )?;
        Ok(())
    }

    pub fn list_devices(&self) -> Result<Vec<DeviceRow>, rusqlite::Error> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, hostname, created_at, last_seen, last_sync
             FROM devices
             WHERE revoked_at IS NULL
             ORDER BY last_seen DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(DeviceRow {
                id: r.get(0)?,
                hostname: r.get(1)?,
                created_at: r.get(2)?,
                last_seen: r.get(3)?,
                last_sync: r.get(4)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }
}

fn open_connection(db_path: &Path) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(db_path)?;
    conn.busy_timeout(std::time::Duration::from_secs(10))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    Ok(conn)
}

fn epoch_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn epoch_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

pub(crate) fn valid_media_hash(hash: &str) -> bool {
    hash.len() == "sha256:".len() + 64
        && hash
            .strip_prefix("sha256:")
            .is_some_and(|suffix| suffix.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

#[cfg(test)]
mod tests {
    use super::{valid_media_hash, Storage};

    #[test]
    fn validates_media_hashes_before_building_blob_paths() {
        let root = std::env::temp_dir().join(format!(
            "aether-sync-storage-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("tempdir");
        let storage = Storage::new(&root.join("sync.db"), &root).expect("storage should open");
        let valid_hash = format!("sha256:{}", "1".repeat(64));

        storage
            .put_blob(&valid_hash, b"encrypted")
            .expect("valid hash should be accepted");
        assert!(storage.has_blob(&valid_hash));
        assert!(storage.get_blob(&valid_hash).expect("read").is_some());

        assert!(!storage.has_blob("../sync.db"));
        assert!(storage.put_blob("../sync.db", b"bad").is_err());
        assert!(storage.get_blob("../sync.db").is_err());
        assert!(valid_media_hash(&valid_hash));
        assert!(!valid_media_hash("sha256:abc"));
        drop(storage);
        let _ = std::fs::remove_dir_all(root);
    }
}
