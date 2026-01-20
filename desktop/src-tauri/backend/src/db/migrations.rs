// Migration runner for database schema migrations
use crate::error::{AppError, Result};
use libsql::Database;
use std::fs;
use std::path::PathBuf;

/// Find the migrations directory
/// 
/// The app is started from desktop/ directory, and when Tauri runs:
/// - Working directory is typically desktop/src-tauri/
/// - Migrations are at desktop/src-tauri/backend/migrations/
fn find_migrations_directory() -> PathBuf {
    // Try 1: Relative to current working directory (most common in dev)
    // When running from src-tauri/, this resolves to ./backend/migrations
    let relative_path = PathBuf::from("./backend/migrations");
    if relative_path.exists() {
        return relative_path;
    }
    
    // Try 2: Relative to executable (for production builds)
    // Executable might be in target/debug/ or target/release/
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Try: exe_dir/../backend/migrations (if exe is in target/debug/)
            let parent_backend = exe_dir.parent()
                .map(|p| p.join("backend").join("migrations"));
            if let Some(ref path) = parent_backend {
                if path.exists() {
                    return path.clone();
                }
            }
            
            // Try: exe_dir/backend/migrations (if exe is in src-tauri/)
            let exe_backend = exe_dir.join("backend").join("migrations");
            if exe_backend.exists() {
                return exe_backend;
            }
        }
    }
    
    // Try 3: Compile-time path (using CARGO_MANIFEST_DIR)
    // This points to desktop/src-tauri/backend/ at compile time
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_migrations = PathBuf::from(manifest_dir).join("migrations");
    if manifest_migrations.exists() {
        return manifest_migrations;
    }
    
    // Fallback: return the most likely path (will be checked for existence by caller)
    PathBuf::from("./backend/migrations")
}

/// Run all pending migrations from SQL files
pub async fn run_migrations(database: &Database) -> Result<()> {
    // Ensure schema_migrations table exists
    let conn = database.connect().map_err(|e| AppError::LibSQL(e))?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        )",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    // Find migrations directory
    // Execution context: app is started from desktop/ directory via `bun tauri dev`
    // When Tauri runs, the working directory is typically desktop/src-tauri/
    // Migrations are located at: desktop/src-tauri/backend/migrations/
    
    use std::path::PathBuf;
    
    // Strategy: Try paths in order of likelihood
    // 1. ./backend/migrations - when running from src-tauri/ (most common in dev)
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
    let mut applied_versions = std::collections::HashSet::new();
    
    let mut rows = conn
        .query("SELECT version FROM schema_migrations ORDER BY version", libsql::params![])
        .await
        .map_err(|e| AppError::LibSQL(e))?;

    while let Ok(Some(row)) = rows.next().await {
        if let Ok(version) = row.get::<String>(0) {
            applied_versions.insert(version);
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

        // Split by semicolons and filter empty statements
        let statements: Vec<&str> = cleaned_sql
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for (idx, statement) in statements.iter().enumerate() {
            // Log the first few characters of each statement for debugging
            let stmt_preview = if statement.len() > 60 {
                format!("{}...", &statement[..60])
            } else {
                statement.to_string()
            };
            tracing::debug!("Executing statement {}: {}", idx + 1, stmt_preview);
            
            // Use execute() for DDL statements (CREATE TABLE, CREATE INDEX, etc.)
            // These don't return rows, so execute() is appropriate
            conn.execute(statement, libsql::params![])
                .await
                .map_err(|e| {
                    tracing::error!("Failed to execute statement {}: {}", idx + 1, statement);
                    let _ = conn.execute("ROLLBACK", libsql::params![]);
                    AppError::LibSQL(e)
                })?;
        }

        // Record migration
        let applied_at = chrono::Utc::now();
        conn.execute(
            "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
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
