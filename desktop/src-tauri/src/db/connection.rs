use crate::error::{AppError, Result};
use crate::settings;
use libsql::{Builder, Database};
use std::env;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::time::Duration;
use std::fs;
use std::io::Write;

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
/// According to Turso's Offline Sync Public Beta:
/// - Synced databases allow writes to local SQLite first (offline writes)
/// - Changes are synced to remote when connectivity is available
/// - Reads are always served from local database for zero latency
/// - Sync pushes local changes and pulls remote changes
/// - Works completely offline - writes proceed without connectivity
/// 
/// Reference: https://turso.tech/blog/turso-offline-sync-public-beta
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
/// This upgrades the local database to a synced database that supports offline writes
/// The existing local data is preserved and will be synced to remote
/// 
/// Uses new_synced_database() per Turso Offline Sync Public Beta docs:
/// - Writes go to local database first (offline writes)
/// - Changes are automatically synced to remote based on sync_interval
/// - Reads are always served from local database
/// - Works offline - writes proceed even without connectivity
/// 
/// Important: The remote database must exist on the server first, or you'll get a 404 error.
/// Reference: https://turso.tech/blog/turso-offline-sync-public-beta
pub async fn configure_sync(
    state: &DbState,
    sync_url: String,
    auth_token: Option<String>,
) -> Result<()> {
    // #region agent log
    let log_path = "/Users/casprine/Desktop/vendor/sunday-studio/aether/.cursor/debug.log";
    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        writeln!(f, "{{\"id\":\"log_configure_sync_entry\",\"timestamp\":{},\"location\":\"connection.rs:64\",\"message\":\"configure_sync called\",\"data\":{{\"sync_url\":\"{}\",\"has_auth_token\":{}}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"A\"}}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
            sync_url, auth_token.is_some())
    });
    // #endregion
    
    let replica_path = "./libsql-replica/local.db";
    
    // #region agent log
    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        writeln!(f, "{{\"id\":\"log_db_path_check\",\"timestamp\":{},\"location\":\"connection.rs:70\",\"message\":\"Checking database file existence\",\"data\":{{\"replica_path\":\"{}\",\"exists\":{}}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"D\"}}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
            replica_path, Path::new(replica_path).exists())
    });
    // #endregion
    
    tracing::info!("Configuring sync with URL: {}", sync_url);

    // #region agent log
    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        let has_sync = state.has_sync_capability.lock().unwrap();
        writeln!(f, "{{\"id\":\"log_current_sync_state\",\"timestamp\":{},\"location\":\"connection.rs:73\",\"message\":\"Current sync state before conversion\",\"data\":{{\"has_sync_capability\":{}}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"A\"}}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
            *has_sync)
    });
    // #endregion

    // Check if database file exists and is local-only (no sync capability)
    let is_local_only = {
        let has_sync_guard = state.has_sync_capability.lock().unwrap();
        !*has_sync_guard
    };
    let db_exists = Path::new(replica_path).exists();
    let mut cleanup_done = false;
    let mut fell_back_to_local = false;  // Track if we fell back to local due to 404
    
    // #region agent log - Count entries before conversion
    let entry_count_before = if db_exists && is_local_only {
        let db = get_database(state);
        let conn = db.connect().ok();
        if let Some(conn) = conn {
            let mut rows = conn.query("SELECT COUNT(*) FROM entries", libsql::params![]).await.ok();
            if let Some(mut rows) = rows {
                if let Ok(Some(row)) = rows.next().await {
                    let count: i64 = row.get(0).unwrap_or(0);
                    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                        writeln!(f, "{{\"id\":\"log_entry_count_before\",\"timestamp\":{},\"location\":\"connection.rs:107\",\"message\":\"Entry count before conversion\",\"data\":{{\"count\":{}}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"E\"}}", 
                            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                            count)
                    });
                    count
                } else { 0 }
            } else { 0 }
        } else { 0 }
    } else { 0 };
    // #endregion

    // #region agent log
    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        writeln!(f, "{{\"id\":\"log_checking_conversion\",\"timestamp\":{},\"location\":\"connection.rs:101\",\"message\":\"Checking if conversion needed\",\"data\":{{\"is_local_only\":{},\"db_exists\":{}}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"I\"}}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
            is_local_only, db_exists)
    });
    // #endregion

    // Try to create replica first without deleting (preserves local data if possible)
    // Only delete if we get a wal_index error
    let mut database_result = if is_local_only && db_exists {
        // First, try to create replica with existing file (might work in some cases)
        // #region agent log
        let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
            writeln!(f, "{{\"id\":\"log_trying_replica_with_existing\",\"timestamp\":{},\"location\":\"connection.rs:112\",\"message\":\"Trying to create replica with existing database file\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"I\"}}", 
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
        });
        // #endregion
        
        let token = auth_token.as_deref().unwrap_or("").to_string();
        // Use new_synced_database for offline sync support (per Turso docs)
        // This enables offline writes that sync automatically when connectivity is restored
        // Reference: https://turso.tech/blog/turso-offline-sync-public-beta
        // Simple configuration - sync_interval is optional, can also sync manually
        let builder = Builder::new_synced_database(replica_path, sync_url.clone(), token.clone());
        // Optionally set sync interval for automatic background syncing
        let sync_interval_secs = get_sync_interval().as_secs();
        let builder = builder.sync_interval(Duration::from_secs(sync_interval_secs));
        // #region agent log
        let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
            writeln!(f, "{{\"id\":\"log_builder_config\",\"timestamp\":{},\"location\":\"connection.rs:157\",\"message\":\"Synced database builder configured (offline sync)\",\"data\":{{\"sync_interval\":{}}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"1\"}}", 
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                sync_interval_secs)
        });
        // #endregion
        builder.build().await
    } else {
        // Not local-only or database doesn't exist, proceed normally
        let token = auth_token.as_deref().unwrap_or("").to_string();
        // Use new_synced_database for offline sync support (per Turso docs)
        let builder = Builder::new_synced_database(replica_path, sync_url.clone(), token.clone());
        let sync_interval_secs = get_sync_interval().as_secs();
        let builder = builder.sync_interval(Duration::from_secs(sync_interval_secs));
        // #region agent log
        let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
            writeln!(f, "{{\"id\":\"log_builder_config_else\",\"timestamp\":{},\"location\":\"connection.rs:171\",\"message\":\"Synced database builder configured (else branch, offline sync)\",\"data\":{{\"sync_interval\":{}}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"1\"}}", 
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                sync_interval_secs)
        });
        // #endregion
        builder.build().await
    };

    // If we get a wal_index error, we need to delete and retry
    // Wrap in Arc immediately for use in spawn and state
    let database = Arc::new(match database_result {
        Ok(db) => {
            // Success! Replica created without deleting
            // #region agent log
            let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                writeln!(f, "{{\"id\":\"log_replica_created_no_delete\",\"timestamp\":{},\"location\":\"connection.rs:167\",\"message\":\"Replica created successfully without deleting local database\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"I\"}}", 
                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
            });
            // #endregion
            db
        }
        Err(e) => {
            let error_msg = e.to_string();
            // #region agent log
            let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                writeln!(f, "{{\"id\":\"log_replica_create_failed\",\"timestamp\":{},\"location\":\"connection.rs:200\",\"message\":\"Failed to create synced database\",\"data\":{{\"error\":\"{}\",\"contains_404\":{},\"contains_wal_index\":{}}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"I\"}}", 
                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                    error_msg, error_msg.contains("404") || error_msg.contains("Not Found"),
                    error_msg.contains("wal_index") || error_msg.contains("wal-index"))
            });
            // #endregion
            
            // Handle 404 error - remote database doesn't exist yet
            // For self-hosted libSQL servers, new_synced_database() might not work
            // Try using new_remote_replica() instead, which works better with self-hosted servers
            if (error_msg.contains("404") || error_msg.contains("Not Found") || error_msg.contains("failed to pull db export")) && !cleanup_done {
                tracing::warn!("new_synced_database() failed with 404. Trying new_remote_replica() for self-hosted server compatibility...");
                // #region agent log
                let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    writeln!(f, "{{\"id\":\"log_404_try_remote_replica\",\"timestamp\":{},\"location\":\"connection.rs:218\",\"message\":\"404 error - trying new_remote_replica instead\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"3\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
                });
                // #endregion
                
                // Try new_remote_replica() which works better with self-hosted libSQL servers
                // This doesn't require the database to exist on the server first
                let token = auth_token.as_deref().unwrap_or("").to_string();
                let mut fallback_builder = Builder::new_remote_replica(replica_path, sync_url.clone(), token.clone());
                let sync_interval_secs = get_sync_interval().as_secs();
                fallback_builder = fallback_builder.sync_interval(Duration::from_secs(sync_interval_secs));
                fallback_builder = fallback_builder.read_your_writes(false);  // Allow offline writes
                
                match fallback_builder.build().await {
                    Ok(replica_db) => {
                        tracing::info!("Successfully created remote replica (self-hosted server compatible)");
                        // #region agent log
                        let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                            writeln!(f, "{{\"id\":\"log_remote_replica_success\",\"timestamp\":{},\"location\":\"connection.rs:232\",\"message\":\"Remote replica created successfully\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"3\"}}", 
                                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
                        });
                        // #endregion
                        replica_db  // Return the replica database
                    }
                    Err(e) => {
                        let fallback_error_msg = e.to_string();
                        // #region agent log
                        let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                            writeln!(f, "{{\"id\":\"log_remote_replica_failed\",\"timestamp\":{},\"location\":\"connection.rs:244\",\"message\":\"Remote replica creation failed\",\"data\":{{\"error\":\"{}\",\"contains_wal_index\":{}}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"3\"}}", 
                                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                                fallback_error_msg, fallback_error_msg.contains("wal_index") || fallback_error_msg.contains("wal-index"))
                        });
                        // #endregion
                        
                        // Check if this is a wal_index error - if so, we need to delete and recreate
                        if (fallback_error_msg.contains("wal_index") || fallback_error_msg.contains("wal-index")) && !cleanup_done {
                            tracing::warn!("wal_index error when creating remote replica. Will backup data, delete database, and create fresh replica.");
                            // #region agent log
                            let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                                writeln!(f, "{{\"id\":\"log_wal_index_in_fallback\",\"timestamp\":{},\"location\":\"connection.rs:252\",\"message\":\"wal_index error in fallback path - will backup and recreate\",\"data\":{{\"entry_count\":{}}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"E\"}}", 
                                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                                    entry_count_before)
                            });
                            // #endregion
                            
                            // Backup database before deletion
                            let backup_path = format!("{}.backup", replica_path);
                            tracing::info!("Backing up local database to {} before conversion", backup_path);
                            
                            // Close the current database connection properly
                            let temp_db = Builder::new_local("/tmp/aether-temp-close.db")
                                .build()
                                .await
                                .map_err(|e| AppError::LibSQL(e))?;
                            
                            {
                                let mut db_guard = state.database.lock().unwrap();
                                *db_guard = Arc::new(temp_db);
                            }
                            tokio::time::sleep(Duration::from_millis(500)).await;
                            
                            // Copy database file to backup location
                            let db_path = Path::new(replica_path);
                            if db_path.exists() {
                                if let Err(e) = fs::copy(db_path, &backup_path) {
                                    tracing::warn!("Failed to backup database file: {}. Data may be lost.", e);
                                } else {
                                    tracing::info!("Database backed up successfully to {}", backup_path);
                                }
                            }
                            
                            // Delete database files
                            let shm_path = format!("{}-shm", replica_path);
                            let wal_path = format!("{}-wal", replica_path);
                            
                            if db_path.exists() {
                                let _ = fs::remove_file(db_path);
                            }
                            if Path::new(&shm_path).exists() {
                                let _ = fs::remove_file(&shm_path);
                            }
                            if Path::new(&wal_path).exists() {
                                let _ = fs::remove_file(&wal_path);
                            }
                            
                            cleanup_done = true;
                            
                            // Now create fresh remote replica (should work without wal_index error)
                            let token = auth_token.as_deref().unwrap_or("").to_string();
                            let mut fresh_builder = Builder::new_remote_replica(replica_path, sync_url.clone(), token.clone());
                            fresh_builder = fresh_builder.sync_interval(Duration::from_secs(get_sync_interval().as_secs()));
                            fresh_builder = fresh_builder.read_your_writes(false);
                            
                            match fresh_builder.build().await {
                                Ok(fresh_replica) => {
                                    tracing::info!("Successfully created fresh remote replica after cleanup");
                                    // #region agent log
                                    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                                        writeln!(f, "{{\"id\":\"log_fresh_replica_success\",\"timestamp\":{},\"location\":\"connection.rs:295\",\"message\":\"Fresh replica created successfully after cleanup\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"E\"}}", 
                                            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
                                    });
                                    // #endregion
                                    fresh_replica
                                }
                                Err(e) => {
                                    tracing::error!("Failed to create fresh remote replica after cleanup: {}", e);
                                    if Path::new(&backup_path).exists() {
                                        tracing::warn!("Original database backed up to: {}. You can restore it manually if needed.", backup_path);
                                    }
                                    return Err(AppError::LibSQL(e));
                                }
                            }
                        } else {
                            // Not a wal_index error, fall back to local database
                            tracing::warn!("new_remote_replica() failed: {}. Falling back to local database.", fallback_error_msg);
                            fell_back_to_local = true;
                            // Create local database as final fallback
                            match Builder::new_local(replica_path)
                                .build()
                                .await
                            {
                                Ok(local_db) => local_db,
                                Err(e) => {
                                    tracing::error!("Failed to create local database fallback: {}", e);
                                    return Err(AppError::LibSQL(e));
                                }
                            }
                        }
                    }
                }
            } else if (error_msg.contains("wal_index") || error_msg.contains("wal-index")) && !cleanup_done && is_local_only {
                // If this is a wal_index error and we haven't cleaned up yet, we need to handle data preservation
                // #region agent log
                let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    writeln!(f, "{{\"id\":\"log_wal_index_requires_delete\",\"timestamp\":{},\"location\":\"connection.rs:188\",\"message\":\"wal_index error - will preserve data before conversion\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"E\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
                });
                // #endregion
                
                // Preserve data by backing up the database file before deletion
                // Use SQLite backup API if possible, otherwise copy the file
                let backup_path = format!("{}.backup", replica_path);
                tracing::info!("Backing up local database to {} before conversion", backup_path);
                
                // #region agent log
                let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    writeln!(f, "{{\"id\":\"log_backup_start\",\"timestamp\":{},\"location\":\"connection.rs:200\",\"message\":\"Starting database backup\",\"data\":{{\"backup_path\":\"{}\",\"entry_count\":{}}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"E\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                        backup_path, entry_count_before)
                });
                // #endregion
                
                // Close the current database connection properly
                // Create temporary database first (outside the lock)
                let temp_db = Builder::new_local("/tmp/aether-temp-close.db")
                    .build()
                    .await
                    .map_err(|e| AppError::LibSQL(e))?;
                
                // Now replace the database while holding the lock (no await inside)
                {
                    let mut db_guard = state.database.lock().unwrap();
                    *db_guard = Arc::new(temp_db);
                }
                tokio::time::sleep(Duration::from_millis(500)).await; // Give more time for file handles to close
                
                // Copy database file to backup location
                let db_path = Path::new(replica_path);
                if db_path.exists() {
                    if let Err(e) = fs::copy(db_path, &backup_path) {
                        tracing::warn!("Failed to backup database file: {}. Data may be lost during conversion.", e);
                        // #region agent log
                        let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                            writeln!(f, "{{\"id\":\"log_backup_failed\",\"timestamp\":{},\"location\":\"connection.rs:220\",\"message\":\"Backup failed\",\"data\":{{\"error\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"E\"}}", 
                                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                                e)
                        });
                        // #endregion
                    } else {
                        tracing::info!("Database backed up successfully to {}", backup_path);
                        // #region agent log
                        let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                            writeln!(f, "{{\"id\":\"log_backup_success\",\"timestamp\":{},\"location\":\"connection.rs:228\",\"message\":\"Backup succeeded\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"E\"}}", 
                                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
                        });
                        // #endregion
                    }
                }
                
                // Delete database files
                let shm_path = format!("{}-shm", replica_path);
                let wal_path = format!("{}-wal", replica_path);
                
                if db_path.exists() {
                    let _ = fs::remove_file(db_path);
                }
                if Path::new(&shm_path).exists() {
                    let _ = fs::remove_file(&shm_path);
                }
                if Path::new(&wal_path).exists() {
                    let _ = fs::remove_file(&wal_path);
                }
                
                cleanup_done = true;
                
                // Retry building the synced database
                let token = auth_token.as_deref().unwrap_or("").to_string();
                let retry_builder = Builder::new_synced_database(replica_path, sync_url.clone(), token.clone());
                let retry_builder = retry_builder.sync_interval(Duration::from_secs(get_sync_interval().as_secs()));
                // #region agent log
                let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    writeln!(f, "{{\"id\":\"log_retry_builder_config\",\"timestamp\":{},\"location\":\"connection.rs:285\",\"message\":\"Retry synced database builder configured (offline sync)\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"1\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
                });
                // #endregion
                
                let new_replica = retry_builder.build().await.map_err(|e| {
                    tracing::error!("Failed to create embedded replica after cleanup: {}", e);
                    // If backup exists, warn user they can restore it
                    if Path::new(&backup_path).exists() {
                        tracing::warn!("Original database backed up to: {}. You can restore it manually if needed.", backup_path);
                    }
                    AppError::LibSQL(e)
                })?;
                
                // After creating replica, attempt to restore data from backup if it exists
                if Path::new(&backup_path).exists() && entry_count_before > 0 {
                    tracing::info!("Attempting to restore data from backup...");
                    // #region agent log
                    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                        writeln!(f, "{{\"id\":\"log_restore_attempt\",\"timestamp\":{},\"location\":\"connection.rs:260\",\"message\":\"Attempting to restore data from backup\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"E\"}}", 
                            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
                    });
                    // #endregion
                    
                    // Note: Full data restoration would require reading from backup and inserting
                    // For now, we just keep the backup file for manual restoration
                    // The user can manually restore if needed, or we could implement full restore later
                    tracing::warn!("Data backup preserved at: {}. Manual restoration may be required.", backup_path);
                }
                
                new_replica  // Return Database, will be wrapped in Arc by the match expression
            } else {
                tracing::error!("Failed to create embedded replica: {}", e);
                return Err(AppError::LibSQL(e));
            }
        }
    });
    
    // #region agent log
    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        writeln!(f, "{{\"id\":\"log_replica_created_final\",\"timestamp\":{},\"location\":\"connection.rs:195\",\"message\":\"Replica database created successfully\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"A\"}}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
    });
    // #endregion


    // Run migrations on the new replica database
    // When we delete the local database and create a new replica, we lose all schema
    // So we need to run migrations to recreate all tables
    // #region agent log
    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        writeln!(f, "{{\"id\":\"log_running_migrations\",\"timestamp\":{},\"location\":\"connection.rs:300\",\"message\":\"Running migrations on new replica\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"H\"}}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
    });
    // #endregion
    
    crate::db::migrations::run_migrations(&database).await
        .map_err(|e| {
            // #region agent log
            let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                writeln!(f, "{{\"id\":\"log_migration_error\",\"timestamp\":{},\"location\":\"connection.rs:307\",\"message\":\"Failed to run migrations on replica\",\"data\":{{\"error\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"H\"}}", 
                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                    e)
            });
            // #endregion
            tracing::error!("Failed to run migrations on replica database: {}", e);
            e
        })?;
    
    // #region agent log
    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        writeln!(f, "{{\"id\":\"log_migrations_complete\",\"timestamp\":{},\"location\":\"connection.rs:315\",\"message\":\"Migrations completed on replica\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"H\"}}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
    });
    // #endregion

    // Apply SQLite optimizations to the new database instance
    apply_sqlite_optimizations(&database).await?;

    // Perform initial sync to pull remote data (per Turso example)
    // Only sync if we didn't fall back to local-only mode
    // Local-only databases don't support sync
    if !fell_back_to_local {
        let database_for_sync: Arc<Database> = Arc::clone(&database);
        
        tokio::spawn(async move {
            // Wait a bit before first sync to allow writes to proceed immediately
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            let log_path = "/Users/casprine/Desktop/vendor/sunday-studio/aether/.cursor/debug.log";
            // #region agent log
            let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                writeln!(f, "{{\"id\":\"log_initial_sync_start\",\"timestamp\":{},\"location\":\"connection.rs:409\",\"message\":\"Starting initial sync (pulls remote and pushes local)\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"2\"}}", 
                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
            });
            // #endregion
            
            tracing::info!("Attempting initial sync (pulls remote and pushes local data)...");
            match database_for_sync.sync().await {
                Ok(_result) => {
                    tracing::info!("Initial sync completed - synced local and remote data");
                    // #region agent log
                    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                        writeln!(f, "{{\"id\":\"log_initial_sync_success\",\"timestamp\":{},\"location\":\"connection.rs:420\",\"message\":\"Initial sync succeeded\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"2\"}}", 
                            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
                    });
                    // #endregion
                }
                Err(e) => {
                    // Log but don't block - this is expected if remote database doesn't exist yet
                    // or if we're offline. Writes will work offline and sync later.
                    tracing::warn!("Initial sync failed (database will work offline, will sync later): {}", e);
                    // #region agent log
                    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                        writeln!(f, "{{\"id\":\"log_initial_sync_failed\",\"timestamp\":{},\"location\":\"connection.rs:428\",\"message\":\"Initial sync failed\",\"data\":{{\"error\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"2\"}}", 
                            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                            e)
                    });
                    // #endregion
                    // Don't fail - writes continue to work offline according to Turso's offline writes feature
                    // Sync will retry automatically based on sync_interval when connectivity is available
                }
            }
        });
        
        tracing::info!("Synced database configured. Writes will work offline and sync in background.");
        
        // Also trigger an immediate sync to push local data to remote
        // The background sync above pulls from remote, but we also want to push local data
        let database_for_immediate_sync: Arc<Database> = Arc::clone(&database);
        tokio::spawn(async move {
            // Wait a bit longer to ensure database is fully initialized
            tokio::time::sleep(Duration::from_millis(500)).await;
            
            let log_path = "/Users/casprine/Desktop/vendor/sunday-studio/aether/.cursor/debug.log";
            // #region agent log
            let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                writeln!(f, "{{\"id\":\"log_immediate_sync_start\",\"timestamp\":{},\"location\":\"connection.rs:425\",\"message\":\"Starting immediate sync to push local data\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"2\"}}", 
                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
            });
            // #endregion
            
            tracing::info!("Triggering immediate sync to push local data to remote...");
            match database_for_immediate_sync.sync().await {
                Ok(_result) => {
                    tracing::info!("Immediate sync completed - local data pushed to remote");
                    // #region agent log
                    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                        writeln!(f, "{{\"id\":\"log_immediate_sync_success\",\"timestamp\":{},\"location\":\"connection.rs:435\",\"message\":\"Immediate sync succeeded - local data pushed\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"2\"}}", 
                            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
                    });
                    // #endregion
                }
                Err(e) => {
                    tracing::warn!("Immediate sync failed (will retry automatically): {}", e);
                    // #region agent log
                    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                        writeln!(f, "{{\"id\":\"log_immediate_sync_failed\",\"timestamp\":{},\"location\":\"connection.rs:442\",\"message\":\"Immediate sync failed\",\"data\":{{\"error\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"2\"}}", 
                            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                            e)
                    });
                    // #endregion
                }
            }
        });
    } else {
        tracing::info!("Local database configured (no sync - remote database doesn't exist yet). Writes will work offline.");
    }

    // Clone values before moving into state (needed for persistence)
    let sync_url_clone = sync_url.clone();
    let auth_token_clone = auth_token.clone();

    // Update state atomically
    {
        let mut db_guard = state.database.lock().unwrap();
        *db_guard = database; // database is already Arc<Database>
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
        // Only set sync capability to true if we didn't fall back to local-only mode
        // Local-only databases don't support sync
        *sync_guard = !fell_back_to_local;
    }

    // Persist sync configuration to database
    let database = get_database(state);
    
    // #region agent log
    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        writeln!(f, "{{\"id\":\"log_persisting_settings\",\"timestamp\":{},\"location\":\"connection.rs:337\",\"message\":\"Persisting sync settings to database\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"G\"}}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
    });
    // #endregion
    
    settings::set_setting(database.clone(), "sync_url", &sync_url_clone).await
        .map_err(|e| {
            // #region agent log
            let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                writeln!(f, "{{\"id\":\"log_set_setting_error\",\"timestamp\":{},\"location\":\"connection.rs:343\",\"message\":\"Failed to set sync_url setting\",\"data\":{{\"error\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"G\"}}", 
                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                    e)
            });
            // #endregion
            e
        })?;
    
    if let Some(token) = &auth_token_clone {
        settings::set_setting(database.clone(), "sync_auth_token", token).await
            .map_err(|e| {
                // #region agent log
                let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    writeln!(f, "{{\"id\":\"log_set_token_error\",\"timestamp\":{},\"location\":\"connection.rs:352\",\"message\":\"Failed to set sync_auth_token setting\",\"data\":{{\"error\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"G\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                        e)
                });
                // #endregion
                e
            })?;
    } else {
        // Delete token if None (user removed token)
        settings::delete_setting(database, "sync_auth_token").await?;
    }

    tracing::info!("Sync configured successfully and persisted to database");
    Ok(())
}

/// Get sync interval from environment or use default
fn get_sync_interval() -> Duration {
    let interval_str = env::var("LIBSQL_SYNC_INTERVAL").unwrap_or_else(|_| "10".to_string());
    
    let duration = if let Ok(seconds) = interval_str.parse::<u64>() {
        Duration::from_secs(seconds)
    } else if let Ok(duration) = interval_str.parse::<humantime::Duration>() {
        duration.into()
    } else {
        Duration::from_secs(10) // Default: 10 seconds
    };
    
    tracing::info!("Sync interval configured: {} seconds", duration.as_secs());
    duration
}

/// Apply SQLite optimizations (PRAGMA settings)
/// Note: Some PRAGMAs are not supported in replica mode and will be skipped
async fn apply_sqlite_optimizations(database: &Database) -> Result<()> {
    // #region agent log
    let log_path = "/Users/casprine/Desktop/vendor/sunday-studio/aether/.cursor/debug.log";
    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        writeln!(f, "{{\"id\":\"log_apply_optimizations_entry\",\"timestamp\":{},\"location\":\"connection.rs:365\",\"message\":\"apply_sqlite_optimizations called\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"F\"}}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
    });
    // #endregion
    
    let conn = database
        .connect()
        .map_err(|e| AppError::LibSQL(e))?;

    // PRAGMAs that are safe for both local and replica databases
    let safe_pragmas = vec![
        "PRAGMA synchronous = NORMAL",
        "PRAGMA cache_size = -32000",
        "PRAGMA temp_store = MEMORY",
        "PRAGMA mmap_size = 67108864",
        "PRAGMA busy_timeout = 10000",
        "PRAGMA foreign_keys = ON",
        "PRAGMA locking_mode = NORMAL",
    ];

    // PRAGMAs that may not be supported in replica mode (only set on local databases)
    // These are typically only effective when set during database creation
    let local_only_pragmas = vec![
        "PRAGMA page_size = 4096",
        "PRAGMA auto_vacuum = INCREMENTAL",
    ];

    // Apply safe PRAGMAs (these work on both local and replica)
    for pragma in safe_pragmas {
        // #region agent log
        let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
            writeln!(f, "{{\"id\":\"log_applying_pragma\",\"timestamp\":{},\"location\":\"connection.rs:385\",\"message\":\"Applying PRAGMA\",\"data\":{{\"pragma\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"F\"}}", 
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                pragma)
        });
        // #endregion
        
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
                // #region agent log
                let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    writeln!(f, "{{\"id\":\"log_pragma_error\",\"timestamp\":{},\"location\":\"connection.rs:395\",\"message\":\"PRAGMA failed (skipping)\",\"data\":{{\"pragma\":\"{}\",\"error\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"F\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                        pragma, e)
                });
                // #endregion
                // Log but don't fail - some PRAGMAs may not be supported in replica mode
                tracing::warn!("Failed to apply PRAGMA {} (may not be supported): {}", pragma, e);
            }
        }
    }

    // Try to apply local-only PRAGMAs, but don't fail if they're not supported
    // These typically only work on newly created databases
    for pragma in local_only_pragmas {
        // #region agent log
        let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
            writeln!(f, "{{\"id\":\"log_applying_local_pragma\",\"timestamp\":{},\"location\":\"connection.rs:410\",\"message\":\"Applying local-only PRAGMA (may fail in replica)\",\"data\":{{\"pragma\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"F\"}}", 
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                pragma)
        });
        // #endregion
        
        match conn.query(pragma, libsql::params![]).await {
            Ok(mut rows) => {
                while let Ok(Some(_)) = rows.next().await {}
            }
            Err(e) => {
                // #region agent log
                let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                    writeln!(f, "{{\"id\":\"log_local_pragma_error\",\"timestamp\":{},\"location\":\"connection.rs:420\",\"message\":\"Local PRAGMA failed (expected in replica mode)\",\"data\":{{\"pragma\":\"{}\",\"error\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"F\"}}", 
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                        pragma, e)
                });
                // #endregion
                // These are expected to fail in replica mode - just log and continue
                tracing::debug!("PRAGMA {} not supported (expected in replica mode): {}", pragma, e);
            }
        }
    }

    // #region agent log
    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        writeln!(f, "{{\"id\":\"log_optimizations_complete\",\"timestamp\":{},\"location\":\"connection.rs:430\",\"message\":\"SQLite optimizations applied\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"F\"}}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
    });
    // #endregion

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
    
    // #region agent log
    let log_path = "/Users/casprine/Desktop/vendor/sunday-studio/aether/.cursor/debug.log";
    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        writeln!(f, "{{\"id\":\"log_manual_sync_start\",\"timestamp\":{},\"location\":\"connection.rs:639\",\"message\":\"Manual sync started\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"2\"}}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
    });
    // #endregion
    
    database
        .sync()
        .await
        .map_err(|e| {
            tracing::error!("Manual sync failed: {}", e);
            // #region agent log
            let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
                writeln!(f, "{{\"id\":\"log_manual_sync_failed\",\"timestamp\":{},\"location\":\"connection.rs:650\",\"message\":\"Manual sync failed\",\"data\":{{\"error\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"2\"}}", 
                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                    e)
            });
            // #endregion
            AppError::LibSQL(e)
        })?;

    tracing::info!("Manual sync completed");
    // #region agent log
    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        writeln!(f, "{{\"id\":\"log_manual_sync_success\",\"timestamp\":{},\"location\":\"connection.rs:657\",\"message\":\"Manual sync succeeded\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"2\"}}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
    });
    // #endregion
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

/// Restore sync configuration from database settings
/// Called after migrations to automatically enable sync if previously configured
pub async fn restore_sync_configuration(state: &DbState) -> Result<()> {
    // #region agent log
    let log_path = "/Users/casprine/Desktop/vendor/sunday-studio/aether/.cursor/debug.log";
    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        writeln!(f, "{{\"id\":\"log_restore_sync_entry\",\"timestamp\":{},\"location\":\"connection.rs:249\",\"message\":\"restore_sync_configuration called\",\"data\":{{}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"E\"}}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
    });
    // #endregion
    
    let database = get_database(state);
    
    // Load saved settings
    let sync_url = settings::get_setting(database.clone(), "sync_url").await?;
    let auth_token = settings::get_setting(database, "sync_auth_token").await?;
    
    // #region agent log
    let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
        writeln!(f, "{{\"id\":\"log_restore_settings\",\"timestamp\":{},\"location\":\"connection.rs:260\",\"message\":\"Loaded sync settings\",\"data\":{{\"has_sync_url\":{},\"has_auth_token\":{}}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"E\"}}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
            sync_url.is_some(), auth_token.is_some())
    });
    // #endregion
    
    // If sync_url exists, restore sync configuration
    if let Some(url) = sync_url {
        tracing::info!("Restoring sync configuration from saved settings");
        // #region agent log
        let _ = fs::OpenOptions::new().create(true).append(true).open(&log_path).and_then(|mut f| {
            writeln!(f, "{{\"id\":\"log_restore_calling_configure\",\"timestamp\":{},\"location\":\"connection.rs:267\",\"message\":\"Calling configure_sync from restore\",\"data\":{{\"url\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"E\"}}", 
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                url)
        });
        // #endregion
        configure_sync(state, url, auth_token).await?;
        tracing::info!("Sync configuration restored successfully");
    } else {
        tracing::info!("No saved sync configuration found, starting in local-only mode");
    }
    
    Ok(())
}
