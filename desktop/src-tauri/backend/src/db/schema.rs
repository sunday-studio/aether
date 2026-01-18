// Schema creation - creates all database tables
use crate::error::{AppError, Result};
use libsql::Database;

/// Create all database tables and indexes
/// This function creates the complete schema based on the models
pub async fn create_schema(database: &Database) -> Result<()> {
    let conn = database.connect().map_err(|e| AppError::LibSQL(e))?;

    // Settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            id TEXT PRIMARY KEY,
            timezone TEXT NOT NULL DEFAULT 'UTC',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    // Tags table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT
        )",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tags_deleted_at ON tags(deleted_at)",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    // Entries table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS entries (
            id TEXT PRIMARY KEY,
            document TEXT NOT NULL,
            created_at TEXT NOT NULL,
            is_pinned INTEGER NOT NULL DEFAULT 0,
            is_archived INTEGER NOT NULL DEFAULT 0,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL,
            deleted_at TEXT
        )",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_entries_deleted_at ON entries(deleted_at)",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_entries_created_at ON entries(created_at)",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    // Entry-Tag many-to-many relationship
    conn.execute(
        "CREATE TABLE IF NOT EXISTS entry_tags (
            entry_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            PRIMARY KEY (entry_id, tag_id),
            FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        )",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    // Goals table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS goals (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            is_non_recurring INTEGER NOT NULL DEFAULT 0,
            recurrence_type TEXT,
            recurrence_interval INTEGER,
            recurrence_anchor TEXT,
            recurrence_meta TEXT,
            timezone TEXT NOT NULL DEFAULT 'UTC',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT
        )",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_goals_deleted_at ON goals(deleted_at)",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    // Goal-Tag many-to-many relationship
    conn.execute(
        "CREATE TABLE IF NOT EXISTS goal_tags (
            goal_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            PRIMARY KEY (goal_id, tag_id),
            FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        )",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    // Goal Instances table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS goal_instances (
            id TEXT PRIMARY KEY,
            goal_id TEXT NOT NULL,
            period_start TEXT NOT NULL,
            period_end TEXT,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE CASCADE
        )",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_goal_period ON goal_instances(goal_id, period_start)",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_goal_instances_goal_id ON goal_instances(goal_id)",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    // Goal Instance-Tag many-to-many relationship
    conn.execute(
        "CREATE TABLE IF NOT EXISTS goal_instance_tags (
            goal_instance_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            PRIMARY KEY (goal_instance_id, tag_id),
            FOREIGN KEY (goal_instance_id) REFERENCES goal_instances(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        )",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    // Tasks table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            is_completed INTEGER NOT NULL DEFAULT 0,
            due_date TEXT,
            goal_instance_id TEXT,
            goal_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            FOREIGN KEY (goal_instance_id) REFERENCES goal_instances(id) ON DELETE SET NULL,
            FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE SET NULL
        )",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_deleted_at ON tasks(deleted_at)",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date)",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_goal_instance_id ON tasks(goal_instance_id)",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_goal_id ON tasks(goal_id)",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    // Task-Tag many-to-many relationship
    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_tags (
            task_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            PRIMARY KEY (task_id, tag_id),
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        )",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    // Subtasks table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS subtasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            is_completed INTEGER NOT NULL DEFAULT 0,
            task_id TEXT NOT NULL,
            order_index INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
        )",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_subtasks_deleted_at ON subtasks(deleted_at)",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_subtasks_task_id ON subtasks(task_id)",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_subtasks_order_index ON subtasks(task_id, order_index)",
        libsql::params![],
    )
    .await
    .map_err(|e| AppError::LibSQL(e))?;

    tracing::info!("Schema created successfully");
    Ok(())
}
