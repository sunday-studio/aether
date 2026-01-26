use crate::error::{AppError, Result};
use libsql::{Builder, Database};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// #region agent log
pub fn debug_log(location: &str, message: &str, data: serde_json::Value) {
    let log_path = "/Users/casprine/Desktop/vendor/sunday-studio/aether/.cursor/debug.log";
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let entry = serde_json::json!({
        "id": format!("log_{}_{}", timestamp, Uuid::new_v4().to_string().split('-').next().unwrap()),
        "timestamp": timestamp,
        "location": location,
        "message": message,
        "data": data,
        "sessionId": "debug-session",
        "runId": "run1"
    });
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(log_path) {
        use std::io::Write;
        let _ = writeln!(file, "{}", entry);
    }
}

pub fn debug_log_connect(location: &str, thread_id: &str) {
    debug_log(location, "Connection created", serde_json::json!({
        "thread_id": thread_id,
        "hypothesisId": "A"
    }));
}

pub fn debug_log_connect_error(location: &str, error: &str, thread_id: &str) {
    let is_locked = error.contains("database is locked") || error.contains("locked");
    debug_log(location, "Connection error", serde_json::json!({
        "error": error,
        "is_locked": is_locked,
        "thread_id": thread_id,
        "hypothesisId": if is_locked { "A" } else { "B" }
    }));
}
// #endregion

#[derive(Clone)]
pub struct DbState {
    pub database: Arc<Mutex<Arc<Database>>>,
}

/// Initialize the database connection in local-only mode
pub async fn initialize() -> Result<DbState> {
    let db_path = "./libsql-replica/local.db";

    // Ensure database directory exists
    if let Some(parent) = Path::new(db_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Io(e))?;
    }

    tracing::info!("Initializing local database");

    let database = Builder::new_local(db_path)
        .build()
        .await
        .map_err(|e| AppError::LibSQL(e))?;

    // Apply SQLite optimizations
    apply_sqlite_optimizations(&database).await?;

    Ok(DbState {
        database: Arc::new(Mutex::new(Arc::new(database))),
    })
}

/// Get current database instance (for use in handlers and repositories)
pub fn get_database(state: &DbState) -> Arc<Database> {
    // #region agent log
    debug_log("connection.rs:37", "get_database called", serde_json::json!({"thread_id": format!("{:?}", std::thread::current().id())}));
    // #endregion
    let db_guard = state.database.lock().unwrap();
    Arc::clone(&*db_guard)
}

/// Apply SQLite optimizations (PRAGMA settings)
async fn apply_sqlite_optimizations(database: &Database) -> Result<()> {
    let conn = database
        .connect()
        .map_err(|e| AppError::LibSQL(e))?;

    // PRAGMAs for local database optimization
    let pragmas = vec![
        "PRAGMA synchronous = NORMAL",
        "PRAGMA cache_size = -32000",
        "PRAGMA temp_store = MEMORY",
        "PRAGMA mmap_size = 67108864",
        "PRAGMA busy_timeout = 10000",
        "PRAGMA foreign_keys = ON",
        "PRAGMA locking_mode = NORMAL",
        "PRAGMA page_size = 4096",
        "PRAGMA auto_vacuum = INCREMENTAL",
    ];

    // Apply PRAGMAs
    for pragma in pragmas {
        // Use query() instead of execute() for PRAGMA statements
        // Some PRAGMAs can return rows, and LibSQL requires using query() for statements that return rows
        match conn.query(pragma, libsql::params![]).await {
            Ok(mut rows) => {
                // Consume any returned rows (even if empty)
                while let Ok(Some(_)) = rows.next().await {
                    // PRAGMA statements may return rows with the current value
                    // We just need to consume them
                }
            }
            Err(e) => {
                // Log but don't fail - some PRAGMAs may not be supported
                tracing::warn!("Failed to apply PRAGMA {}: {}", pragma, e);
            }
        }
    }

    Ok(())
}
