//! SQLite storage for encrypted changes and blob metadata. Blob files in {data}/blobs/{hash}.

use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

pub struct Storage {
    conn: Mutex<Connection>,
    blob_dir: std::path::PathBuf,
}

impl Storage {
    pub fn new(db_path: &Path, data_root: &Path) -> Result<Self, rusqlite::Error> {
        std::fs::create_dir_all(db_path.parent().unwrap_or(Path::new("."))).ok();
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS changes (
                id INTEGER PRIMARY KEY,
                device_id TEXT NOT NULL,
                nonce BLOB NOT NULL,
                ciphertext BLOB NOT NULL,
                received_at INTEGER NOT NULL
            )",
            [],
        )?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_changes_time ON changes(received_at)", [])?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS blobs (
                hash TEXT PRIMARY KEY,
                size INTEGER NOT NULL,
                uploaded_at INTEGER NOT NULL
            )",
            [],
        )?;

        let blob_dir = data_root.join("blobs");
        std::fs::create_dir_all(&blob_dir).ok();

        Ok(Self {
            conn: Mutex::new(conn),
            blob_dir,
        })
    }

    pub fn push(&self, device_id: &str, nonce: &[u8], ciphertext: &[u8]) -> Result<(), rusqlite::Error> {
        let received_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO changes (device_id, nonce, ciphertext, received_at) VALUES (?1, ?2, ?3, ?4)",
            params![device_id, nonce, ciphertext, received_at],
        )?;
        Ok(())
    }

    pub fn pull(&self, since: i64, limit: i64) -> Result<Vec<(Vec<u8>, Vec<u8>)>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT nonce, ciphertext FROM changes WHERE received_at > ?1 ORDER BY received_at LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![since, limit], |r| {
            Ok((r.get::<_, Vec<u8>>(0)?, r.get::<_, Vec<u8>>(1)?))
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn max_received_at(&self) -> Result<i64, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let v: i64 = conn.query_row(
            "SELECT COALESCE(MAX(received_at), 0) FROM changes",
            [],
            |r| r.get(0),
        )?;
        Ok(v)
    }

    pub fn put_blob(&self, hash: &str, data: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let p = self.blob_dir.join(hash);
        std::fs::write(&p, data)?;
        let conn = self.conn.lock().unwrap();
        let at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        conn.execute(
            "INSERT OR REPLACE INTO blobs (hash, size, uploaded_at) VALUES (?1, ?2, ?3)",
            params![hash, data.len() as i64, at],
        )?;
        Ok(())
    }

    pub fn get_blob(&self, hash: &str) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
        let p = self.blob_dir.join(hash);
        if p.exists() {
            Ok(Some(std::fs::read(&p)?))
        } else {
            Ok(None)
        }
    }

    pub fn has_blob(&self, hash: &str) -> bool {
        self.blob_dir.join(hash).exists()
    }
}
