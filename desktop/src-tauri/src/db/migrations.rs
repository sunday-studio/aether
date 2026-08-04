// Migration runner for database schema migrations
use crate::error::{AppError, Result};
use libsql::Database;

// Migrations must be available in the installed app, where the source-tree
// `migrations/` directory does not exist. Embedding them also makes migration
// behavior independent of the process working directory.
const EMBEDDED_MIGRATIONS: &[(&str, &str)] = &[
    (
        "001_initial_schema",
        include_str!("../../migrations/001_initial_schema.sql"),
    ),
    (
        "002_search_and_fts",
        include_str!("../../migrations/002_search_and_fts.sql"),
    ),
    (
        "003_fts_triggers",
        include_str!("../../migrations/003_fts_triggers.sql"),
    ),
    (
        "004_media_and_transcription",
        include_str!("../../migrations/004_media_and_transcription.sql"),
    ),
    (
        "005_sync_infrastructure",
        include_str!("../../migrations/005_sync_infrastructure.sql"),
    ),
    (
        "006_vector_embeddings",
        include_str!("../../migrations/006_vector_embeddings.sql"),
    ),
    (
        "007_search_documents",
        include_str!("../../migrations/007_search_documents.sql"),
    ),
    (
        "008_search_documents_fts",
        include_str!("../../migrations/008_search_documents_fts.sql"),
    ),
    (
        "009_search_embeddings",
        include_str!("../../migrations/009_search_embeddings.sql"),
    ),
    (
        "010_ai_journal_enrichment",
        include_str!("../../migrations/010_ai_journal_enrichment.sql"),
    ),
    (
        "011_sync_deferred_changes",
        include_str!("../../migrations/011_sync_deferred_changes.sql"),
    ),
];

/// Run all pending migrations from SQL files
pub async fn run_migrations(database: &Database) -> Result<()> {
    // Ensure schema_migrations table exists
    let conn = database.connect().map_err(|e| AppError::LibSQL(e))?;

    // Try to create the schema_migrations table
    match conn
        .execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        )",
            libsql::params![],
        )
        .await
    {
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

    tracing::info!("Using {} embedded migrations", EMBEDDED_MIGRATIONS.len());

    // Get applied migrations
    // Handle the case where the table might not exist yet
    let mut applied_versions = std::collections::HashSet::new();

    match conn
        .query(
            "SELECT version FROM schema_migrations ORDER BY version",
            libsql::params![],
        )
        .await
    {
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
                tracing::debug!(
                    "schema_migrations table doesn't exist yet, treating as no migrations applied"
                );
            } else {
                // Some other error, propagate it
                return Err(AppError::LibSQL(e));
            }
        }
    }

    // Run pending migrations from files
    for (migration_name, sql) in EMBEDDED_MIGRATIONS {
        let version = (*migration_name).to_string();

        if applied_versions.contains(&version) {
            tracing::debug!("Migration {} already applied, skipping", version);
            continue;
        }

        tracing::info!("Running migration: {}", version);

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
                    if (before.is_none() || !before.unwrap().is_alphanumeric())
                        && (after.is_none() || !after.unwrap().is_alphanumeric())
                    {
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
                    if (before.is_none() || !before.unwrap().is_alphanumeric())
                        && (after.is_none() || !after.unwrap().is_alphanumeric())
                    {
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
