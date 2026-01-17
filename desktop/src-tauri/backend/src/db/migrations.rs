// Migration runner for database schema migrations
use crate::error::{AppError, Result};
use libsql::Database;
use std::fs;
use std::path::Path;

/// Run all pending migrations
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

    // Get all migration files
    let migrations_dir = Path::new("./migrations");
    if !migrations_dir.exists() {
        tracing::warn!("Migrations directory not found, skipping migrations");
        return Ok(());
    }

    let mut migration_files: Vec<_> = fs::read_dir(migrations_dir)
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

    // Run pending migrations
    for migration_file in migration_files {
        // Extract version from filename (e.g., "001_initial.sql" -> "001_initial")
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

        // Execute migration in a transaction
        // Note: Some migrations may need to run without transaction (PRAGMA statements)
        // For now, we'll run all in transaction - can be enhanced later
        conn.execute("BEGIN TRANSACTION", libsql::params![])
            .await
            .map_err(|e| AppError::LibSQL(e))?;

        // Split SQL by semicolons and execute each statement
        for statement in sql.split(';') {
            let statement = statement.trim();
            if statement.is_empty() || statement.starts_with("--") {
                continue;
            }

            conn.execute(statement, libsql::params![])
                .await
                .map_err(|e| {
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
