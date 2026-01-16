use crate::error::{AppError, Result};
use libsql::{Builder, Database};
use std::env;
use std::path::Path;
use std::sync::Arc;
use tokio::time::{interval, Duration};

#[derive(Clone)]
pub struct DbState {
    pub database: Arc<Database>,
    pub has_sync_capability: bool,
}

/// Initialize the database connection
/// Supports both local-only mode (no LIBSQL_URL) and embedded replica mode (with LIBSQL_URL)
pub async fn initialize() -> Result<DbState> {
    let libsql_url = env::var("LIBSQL_URL").ok();
    let _auth_token = env::var("LIBSQL_AUTH_TOKEN").ok();
    let replica_path = "./libsql-replica/local.db";

    // Ensure replica directory exists
    if let Some(parent) = Path::new(replica_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Io(e))?;
    }

    let (database, has_sync) = if libsql_url.is_some() {
        // Embedded replica mode
        // Note: The libsql Rust crate API for embedded replicas may differ from Go
        // For now, we'll use local mode and note that sync needs to be implemented
        // based on the actual libsql Rust API when available
        tracing::info!("Initializing embedded replica mode (sync implementation pending)");

        let database = Builder::new_local(replica_path)
            .build()
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        // Start background sync (placeholder for now)
        let sync_interval = get_sync_interval();
        let db_for_sync = Arc::new(database);
        let url = libsql_url.unwrap();
        let token = _auth_token;
        start_background_sync(db_for_sync.clone(), url, token, sync_interval);

        (db_for_sync, true)
    } else {
        // Local-only mode
        tracing::info!("Initializing local-only mode");

        let database = Builder::new_local(replica_path)
            .build()
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        (Arc::new(database), false)
    };

    // Apply SQLite optimizations
    apply_sqlite_optimizations(&database).await?;

    // Run migrations - will be called from main.rs after initialization
    // For now, skip here to avoid circular dependency

    Ok(DbState {
        database,
        has_sync_capability: has_sync,
    })
}

/// Get sync interval from environment or use default
fn get_sync_interval() -> Duration {
    let interval_str = env::var("LIBSQL_SYNC_INTERVAL").unwrap_or_else(|_| "10".to_string());
    
    if let Ok(seconds) = interval_str.parse::<u64>() {
        Duration::from_secs(seconds)
    } else if let Ok(duration) = interval_str.parse::<humantime::Duration>() {
        duration.into()
    } else {
        Duration::from_secs(10) // Default: 10 seconds
    }
}

/// Start background sync task
/// For embedded replicas, this would sync with the remote database
/// TODO: Implement actual sync using libsql replication API when available
fn start_background_sync(
    _database: Arc<Database>,
    _url: String,
    _auth_token: Option<String>,
    interval_duration: Duration,
) {
    tokio::spawn(async move {
        let mut interval = interval(interval_duration);
        loop {
            interval.tick().await;
            // For now, we'll just log - actual sync implementation depends on libsql API
            // In production, you'd use the replication API to sync frames
            // This requires understanding the libsql replication protocol
            tracing::debug!("Background sync tick (sync implementation pending)");
        }
    });
}

/// Apply SQLite optimizations (PRAGMA settings)
async fn apply_sqlite_optimizations(database: &Database) -> Result<()> {
    let conn = database
        .connect()
        .map_err(|e| AppError::LibSQL(e))?;

    let pragmas = vec![
        "PRAGMA synchronous = NORMAL",
        "PRAGMA cache_size = -32000",
        "PRAGMA temp_store = MEMORY",
        "PRAGMA mmap_size = 67108864",
        "PRAGMA page_size = 4096",
        "PRAGMA busy_timeout = 10000",
        "PRAGMA foreign_keys = ON",
        "PRAGMA locking_mode = NORMAL",
        "PRAGMA auto_vacuum = INCREMENTAL",
    ];

    for pragma in pragmas {
        conn.execute(pragma, libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;
    }

    Ok(())
}

/// Manual sync trigger
pub async fn sync_now(state: &DbState) -> Result<u64> {
    if !state.has_sync_capability {
        return Err(AppError::BadRequest(
            "Sync not available: no LIBSQL_URL configured".to_string(),
        ));
    }

    // For embedded replicas, sync would be handled through the replication API
    // This is a placeholder - actual implementation depends on libsql replication API
    tracing::info!("Manual sync triggered (sync implementation pending)");
    Ok(0) // Placeholder - would return actual frames synced
}
