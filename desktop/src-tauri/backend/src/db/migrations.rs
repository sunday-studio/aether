// Migration runner for database schema migrations
use crate::error::{AppError, Result};
use libsql::Database;
use std::fs;
use std::path::Path;

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
