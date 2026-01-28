// Migration runner for database schema migrations
use crate::error::{AppError, Result};
use libsql::Database;
use std::fs;
use std::path::PathBuf;

/// Find the migrations directory
/// 
/// The app is started from desktop/ directory, and when Tauri runs:
/// - Working directory is typically desktop/src-tauri/
/// - Migrations are at desktop/src-tauri/migrations/
fn find_migrations_directory() -> PathBuf {
    // Try 1: Relative to current working directory (most common in dev)
    // When running from src-tauri/, this resolves to ./migrations
    let relative_path = PathBuf::from("./migrations");
    if relative_path.exists() {
        return relative_path;
    }
    
    // Try 2: Relative to executable (for production builds)
    // Executable might be in target/debug/ or target/release/
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Try: exe_dir/../migrations (if exe is in target/debug/)
            let parent_migrations = exe_dir.parent()
                .map(|p| p.join("migrations"));
            if let Some(ref path) = parent_migrations {
                if path.exists() {
                    return path.clone();
                }
            }
            
            // Try: exe_dir/migrations (if exe is in src-tauri/)
            let exe_migrations = exe_dir.join("migrations");
            if exe_migrations.exists() {
                return exe_migrations;
            }
        }
    }
    
    // Try 3: Compile-time path (using CARGO_MANIFEST_DIR)
    // This points to desktop/src-tauri/ at compile time
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_migrations = PathBuf::from(manifest_dir).join("migrations");
    if manifest_migrations.exists() {
        return manifest_migrations;
    }
    
    // Fallback: return the most likely path (will be checked for existence by caller)
    PathBuf::from("./migrations")
}

/// Run all pending migrations from SQL files
pub async fn run_migrations(database: &Database) -> Result<()> {
    // Ensure schema_migrations table exists
    let conn = database.connect().map_err(|e| AppError::LibSQL(e))?;
    
    // Try to create the schema_migrations table
    match conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        )",
        libsql::params![],
    )
    .await {
        Ok(_) => {
            tracing::debug!("schema_migrations table created or already exists");
        }
        Err(e) => {
            let error_msg = e.to_string();
            // If CREATE TABLE fails, we'll try to continue anyway
            // The query below will handle the missing table case
            tracing::warn!("Failed to create schema_migrations table: {}", error_msg);
        }
    }

    // Find migrations directory
    // Execution context: app is started from desktop/ directory via `bun tauri dev`
    // When Tauri runs, the working directory is typically desktop/src-tauri/
    // Migrations are located at: desktop/src-tauri/migrations/
    
    use std::path::PathBuf;
    
    // Strategy: Try paths in order of likelihood
    // 1. ./migrations - when running from src-tauri/ (most common in dev)
    // 2. Relative to executable - for production builds
    // 3. Compile-time path - using CARGO_MANIFEST_DIR (fallback)
    
    let migrations_dir = find_migrations_directory();
    
    if !migrations_dir.exists() {
        tracing::warn!(
            "Migrations directory not found at {:?}. Current working directory: {:?}",
            migrations_dir,
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("unknown"))
        );
        return Ok(());
    }
    
    tracing::info!("Using migrations directory: {:?}", migrations_dir);

    let mut migration_files: Vec<_> = fs::read_dir(&migrations_dir)
        .map_err(|e| AppError::Io(e))?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension() == Some(std::ffi::OsStr::new("sql")) {
                    path.file_name()?.to_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
        })
        .collect();

    migration_files.sort();

    // Get applied migrations
    // Handle the case where the table might not exist yet
    let mut applied_versions = std::collections::HashSet::new();
    
    match conn.query("SELECT version FROM schema_migrations ORDER BY version", libsql::params![]).await {
        Ok(mut rows) => {
            while let Ok(Some(row)) = rows.next().await {
                if let Ok(version) = row.get::<String>(0) {
                    applied_versions.insert(version);
                }
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            // If the table doesn't exist, that's fine - we'll treat it as no migrations applied
            if error_msg.contains("no such table") || error_msg.contains("does not exist") {
                tracing::debug!("schema_migrations table doesn't exist yet, treating as no migrations applied");
            } else {
                // Some other error, propagate it
                return Err(AppError::LibSQL(e));
            }
        }
    }

    // Run pending migrations from files
    for migration_file in migration_files {
        // Extract version from filename (e.g., "001_initial_schema.sql" -> "001_initial_schema")
        let version = migration_file
            .strip_suffix(".sql")
            .unwrap_or(&migration_file)
            .to_string();

        if applied_versions.contains(&version) {
            tracing::debug!("Migration {} already applied, skipping", version);
            continue;
        }

        tracing::info!("Running migration: {}", version);

        let migration_path = migrations_dir.join(&migration_file);
        let sql = fs::read_to_string(&migration_path)
            .map_err(|e| AppError::Io(e))?;

        // Skip empty migration files
        if sql.trim().is_empty() {
            tracing::debug!("Migration {} is empty, skipping", version);
            continue;
        }

        // Execute migration in a transaction
        conn.execute("BEGIN TRANSACTION", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        // Split SQL by semicolons and execute each statement
        // First, remove comment lines and inline comments
        let mut cleaned_sql = String::new();
        for line in sql.lines() {
            let trimmed = line.trim();
            // Skip comment-only lines
            if trimmed.starts_with("--") {
                continue;
            }
            // Remove inline comments
            let line_content = if let Some(comment_pos) = trimmed.find("--") {
                &trimmed[..comment_pos]
            } else {
                trimmed
            };
            if !line_content.is_empty() {
                cleaned_sql.push_str(line_content);
                cleaned_sql.push(' ');
            }
        }

        // Split by semicolons, but respect BEGIN...END blocks
        // Track nesting level to avoid splitting inside triggers/procedures
        let mut statements = Vec::new();
        let mut current_statement = String::new();
        let mut begin_count = 0;
        let mut chars = cleaned_sql.chars().peekable();
        
        while let Some(ch) = chars.next() {
            current_statement.push(ch);
            
            // Check for BEGIN keyword (case-insensitive, whole word)
            if current_statement.len() >= 5 {
                let end_pos = current_statement.len();
                let start_pos = end_pos.saturating_sub(5);
                let word = &current_statement[start_pos..end_pos];
                if word.eq_ignore_ascii_case("BEGIN") {
                    // Check word boundaries (not part of another word)
                    let before = if start_pos > 0 {
                        current_statement.chars().nth(start_pos - 1)
                    } else {
                        None
                    };
                    let after = chars.peek();
                    if (before.is_none() || !before.unwrap().is_alphanumeric()) &&
                       (after.is_none() || !after.unwrap().is_alphanumeric()) {
                        begin_count += 1;
                    }
                }
            }
            
            // Check for END keyword (case-insensitive, whole word)
            if current_statement.len() >= 3 {
                let end_pos = current_statement.len();
                let start_pos = end_pos.saturating_sub(3);
                let word = &current_statement[start_pos..end_pos];
                if word.eq_ignore_ascii_case("END") {
                    // Check word boundaries (not part of another word)
                    let before = if start_pos > 0 {
                        current_statement.chars().nth(start_pos - 1)
                    } else {
                        None
                    };
                    let after = chars.peek();
                    if (before.is_none() || !before.unwrap().is_alphanumeric()) &&
                       (after.is_none() || !after.unwrap().is_alphanumeric()) {
                        if begin_count > 0 {
                            begin_count -= 1;
                        }
                    }
                }
            }
            
            // Only split on semicolon if we're not inside a BEGIN...END block
            if ch == ';' && begin_count == 0 {
                let trimmed = current_statement.trim();
                if !trimmed.is_empty() {
                    statements.push(trimmed.to_string());
                }
                current_statement.clear();
            }
        }
        
        // Add any remaining statement
        let trimmed = current_statement.trim();
        if !trimmed.is_empty() {
            statements.push(trimmed.to_string());
        }

        for (idx, statement) in statements.iter().enumerate() {
            // Log the first few characters of each statement for debugging
            let stmt_preview = if statement.len() > 60 {
                format!("{}...", &statement[..60])
            } else {
                statement.to_string()
            };
            tracing::debug!("Executing statement {}: {}", idx + 1, stmt_preview);
            
            // Special handling for vector index creation - these may fail if libsql_vector_idx is not available
            // Allow them to fail gracefully without rolling back the entire migration
            let is_vector_index = statement.to_uppercase().contains("LIBSQL_VECTOR_IDX");
            
            match conn.execute(statement, libsql::params![]).await {
                Ok(_) => {
                    if is_vector_index {
                        tracing::info!("Vector index created successfully");
                    }
                }
                Err(e) => {
                    if is_vector_index {
                        let error_msg = e.to_string();
                        tracing::warn!(
                            "Failed to create vector index (libsql_vector_idx may not be available): {}",
                            error_msg
                        );
                        // Continue without the index - vector search will still work, just slower
                    } else {
                        tracing::error!("Failed to execute statement {}: {}", idx + 1, statement);
                        let _ = conn.execute("ROLLBACK", libsql::params![]);
                        return Err(AppError::LibSQL(e));
                    }
                }
            }
        }

        // Record migration
        // Use INSERT OR IGNORE to handle the case where migration was already applied
        let applied_at = chrono::Utc::now();
        conn.execute(
            "INSERT OR IGNORE INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
            libsql::params![version.clone(), version.clone(), applied_at.to_rfc3339()],
        )
        .await
        .map_err(|e| {
            let _ = conn.execute("ROLLBACK", libsql::params![]);
            AppError::LibSQL(e)
        })?;

        conn.execute("COMMIT", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        tracing::info!("Migration {} completed", version);
    }

    tracing::info!("All migrations completed");
    Ok(())
}
