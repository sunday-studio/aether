use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use libsql::{Builder, Database};
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex as AsyncMutex;

use crate::error::{AppError, Result};

const DB_ACCESS_WAIT_LOG_THRESHOLD: Duration = Duration::from_millis(25);

#[derive(Clone)]
pub struct DbState {
    pub database: Arc<Mutex<Arc<Database>>>,
    /// Serializes all DB access (read and write) so only one operation runs at a time. Prevents "database is locked".
    pub db_access: Arc<AsyncMutex<()>>,
}

/// Database path: local dev = target/libsql-replica-dev (avoids watcher rebuilds); build = app data dir.
fn get_db_path(app_handle: Option<&AppHandle>) -> Result<PathBuf> {
    let app_data_dir = if let Some(handle) = app_handle {
        handle.path().app_data_dir().map_err(|e| {
            AppError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Failed to get app data dir: {}", e),
            ))
        })?
    } else if cfg!(debug_assertions) {
        // Local dev: use target/ so DB files are outside src/ and don't trigger the dev watcher
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("libsql-replica-dev");
        return Ok(dir.join("local.db"));
    } else {
        // Build without handle: resolve app data dir from identifier (com.cas.aether)
        directories::ProjectDirs::from("com.cas", "aether", "com.cas.aether")
            .ok_or_else(|| {
                AppError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Failed to resolve app data directory",
                ))
            })?
            .data_local_dir()
            .to_path_buf()
    };

    Ok(app_data_dir.join("libsql-replica").join("local.db"))
}

/// Initialize the database connection in local-only mode.
/// Pass None for local dev (uses project libsql-replica); pass Some(app_handle) when available (e.g. in setup) for app path.
pub async fn initialize(app_handle: Option<&AppHandle>) -> Result<DbState> {
    let db_path = get_db_path(app_handle)?;

    // Ensure database directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::Io(e))?;
    }

    tracing::info!("Initializing local database at: {}", db_path.display());

    let database = Builder::new_local(db_path)
        .build()
        .await
        .map_err(|e| AppError::LibSQL(e))?;

    // Apply SQLite optimizations
    apply_sqlite_optimizations(&database).await?;

    Ok(DbState {
        database: Arc::new(Mutex::new(Arc::new(database))),
        db_access: Arc::new(AsyncMutex::new(())),
    })
}

/// Acquire exclusive access to the database. Hold the guard for the duration of the operation.
#[track_caller]
pub fn with_db_access(
    state: &DbState,
) -> impl std::future::Future<Output = tokio::sync::MutexGuard<'_, ()>> + '_ {
    let caller = std::panic::Location::caller();
    async move {
        let started = Instant::now();
        let guard = state.db_access.lock().await;
        let waited = started.elapsed();
        if waited >= DB_ACCESS_WAIT_LOG_THRESHOLD {
            tracing::info!(
                "[DB-TIMING] db_access_wait={}ms caller={}:{}",
                waited.as_millis(),
                caller.file(),
                caller.line()
            );
        }
        guard
    }
}

/// Get current database instance (for use in handlers and repositories)
pub fn get_database(state: &DbState) -> Arc<Database> {
    let db_guard = state.database.lock().unwrap();
    Arc::clone(&*db_guard)
}

/// Apply SQLite optimizations (PRAGMA settings)
async fn apply_sqlite_optimizations(database: &Database) -> Result<()> {
    let conn = database.connect().map_err(|e| AppError::LibSQL(e))?;

    // PRAGMAs for local database optimization
    // WAL mode must be first for optimal concurrent read/write performance
    let pragmas = vec![
        "PRAGMA journal_mode = WAL",
        "PRAGMA synchronous = NORMAL",
        "PRAGMA cache_size = -64000",
        "PRAGMA temp_store = MEMORY",
        "PRAGMA mmap_size = 268435456",
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
