use crate::error::{AppError, Result};
use libsql::{Builder, Database};
use std::path::Path;
use std::sync::{Arc, Mutex};

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
