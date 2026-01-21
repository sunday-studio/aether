use crate::error::{AppError, Result};
use libsql::{Builder, Database};
use std::env;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::time::Duration;

#[derive(Clone)]
pub struct DbState {
    pub database: Arc<Mutex<Arc<Database>>>,
    pub sync_url: Arc<Mutex<Option<String>>>,
    pub auth_token: Arc<Mutex<Option<String>>>,
    pub has_sync_capability: Arc<Mutex<bool>>,
}

/// Initialize the database connection
/// Always starts in local-only mode. Sync can be enabled later via configure_sync()
/// 
/// According to Turso's Offline Writes feature:
/// - Embedded replicas allow writes to local SQLite WAL first (offline writes)
/// - Changes are synced to remote when connectivity is available
/// - Reads are always served from local replica for zero latency
/// - Sync pushes local WAL changes and pulls remote changes
/// 
/// Reference: https://turso.tech/blog/introducing-offline-writes-for-turso
pub async fn initialize() -> Result<DbState> {
    let replica_path = "./libsql-replica/local.db";

    // Ensure replica directory exists
    if let Some(parent) = Path::new(replica_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Io(e))?;
    }

    // Always start in local-only mode
    // Sync can be enabled later via configure_sync() API
    tracing::info!("Initializing local-only mode (sync can be enabled via API)");

    let database = Builder::new_local(replica_path)
        .build()
        .await
        .map_err(|e| AppError::LibSQL(e))?;

    // Apply SQLite optimizations
    apply_sqlite_optimizations(&database).await?;

    Ok(DbState {
        database: Arc::new(Mutex::new(Arc::new(database))),
        sync_url: Arc::new(Mutex::new(None)),
        auth_token: Arc::new(Mutex::new(None)),
        has_sync_capability: Arc::new(Mutex::new(false)),
    })
}

/// Configure sync with remote database
/// This upgrades the local database to an embedded replica that syncs with remote
/// The existing local data is preserved and will be synced to remote
/// 
/// According to Turso's Offline Writes:
/// - Writes continue to go to local WAL first (offline writes)
/// - Changes are automatically synced to remote based on sync_interval
/// - Reads are always served from local replica
pub async fn configure_sync(
    state: &DbState,
    sync_url: String,
    auth_token: Option<String>,
) -> Result<()> {
    let replica_path = "./libsql-replica/local.db";
    
    tracing::info!("Configuring sync with URL: {}", sync_url);

    // Create new embedded replica using the same database file
    // This preserves all existing local data
    let token = auth_token.as_deref().unwrap_or("").to_string();
    
    let mut builder = Builder::new_remote_replica(replica_path, sync_url.clone(), token);
    
    // Configure sync interval for automatic background sync
    let sync_interval_secs = get_sync_interval().as_secs();
    builder = builder.sync_interval(Duration::from_secs(sync_interval_secs));
    
    // Enable read-your-writes: local writes are immediately visible
    builder = builder.read_your_writes(true);

    let database = builder
        .build()
        .await
        .map_err(|e| {
            tracing::error!("Failed to create embedded replica: {}", e);
            AppError::LibSQL(e)
        })?;

    // Apply SQLite optimizations to the new database instance
    apply_sqlite_optimizations(&database).await?;

    // Perform initial sync to push local data and pull remote data
    match database.sync().await {
        Ok(_result) => {
            tracing::info!("Initial sync completed");
        }
        Err(e) => {
            tracing::warn!("Initial sync failed (will retry automatically): {}", e);
            // Don't fail - sync will retry automatically
        }
    }

    // Update state atomically
    {
        let mut db_guard = state.database.lock().unwrap();
        *db_guard = Arc::new(database);
    }
    
    {
        let mut url_guard = state.sync_url.lock().unwrap();
        *url_guard = Some(sync_url);
    }
    
    {
        let mut token_guard = state.auth_token.lock().unwrap();
        *token_guard = auth_token;
    }
    
    {
        let mut sync_guard = state.has_sync_capability.lock().unwrap();
        *sync_guard = true;
    }

    tracing::info!("Sync configured successfully");
    Ok(())
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
        // Use query() instead of execute() for PRAGMA statements
        // Some PRAGMAs can return rows, and LibSQL requires using query() for statements that return rows
        let mut rows = conn
            .query(pragma, libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;
        
        // Consume any returned rows (even if empty)
        while let Ok(Some(_)) = rows.next().await {
            // PRAGMA statements may return rows with the current value
            // We just need to consume them
        }
    }

    Ok(())
}

/// Manual sync trigger
/// Pushes local WAL changes to remote and pulls remote changes to local
/// 
/// According to Turso's Offline Writes:
/// - Pushes local WAL frames (offline writes) to remote
/// - Pulls remote WAL frames to local
/// - Returns number of frames synced
/// 
/// Reference: https://turso.tech/blog/introducing-offline-writes-for-turso
pub async fn sync_now(state: &DbState) -> Result<u64> {
    let has_sync = {
        let sync_guard = state.has_sync_capability.lock().unwrap();
        *sync_guard
    };

    if !has_sync {
        return Err(AppError::BadRequest(
            "Sync not available: no sync URL configured. Use /v1/sync/configure to enable sync".to_string(),
        ));
    }

    tracing::info!("Manual sync triggered");
    
    let database = {
        let db_guard = state.database.lock().unwrap();
        Arc::clone(&*db_guard)
    };
    
    database
        .sync()
        .await
        .map_err(|e| {
            tracing::error!("Manual sync failed: {}", e);
            AppError::LibSQL(e)
        })?;

    tracing::info!("Manual sync completed");
    // Note: frames_synced is not accessible from the Replicated struct
    // Return 0 as placeholder - the sync operation succeeded
    Ok(0)
}

/// Get current database instance (for use in handlers and repositories)
/// This ensures repositories always get the latest database instance,
/// even after sync is configured
pub fn get_database(state: &DbState) -> Arc<Database> {
    let db_guard = state.database.lock().unwrap();
    Arc::clone(&*db_guard)
}
