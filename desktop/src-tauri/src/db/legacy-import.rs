use crate::error::{AppError, Result};
use chrono::{DateTime, SecondsFormat, Utc};
use libsql::{Builder, Connection, Database, OpenFlags};
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;

const REQUIRED_TABLES: [&str; 10] = [
    "entries",
    "tags",
    "goals",
    "goal_instances",
    "tasks",
    "sub_tasks",
    "entry_tags",
    "goal_tags",
    "goal_instance_tags",
    "task_tags",
];

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LegacyImportCounts {
    pub entries: u64,
    pub tags: u64,
    pub goals: u64,
    pub goal_instances: u64,
    pub tasks: u64,
    pub subtasks: u64,
    pub entry_tags: u64,
    pub goal_tags: u64,
    pub goal_instance_tags: u64,
    pub task_tags: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyImportPreview {
    pub source_path: String,
    pub counts: LegacyImportCounts,
}

#[derive(Default)]
struct LegacyData {
    entries: Vec<EntryRow>,
    tags: Vec<TagRow>,
    goals: Vec<GoalRow>,
    goal_instances: Vec<GoalInstanceRow>,
    tasks: Vec<TaskRow>,
    subtasks: Vec<SubtaskRow>,
    entry_tags: Vec<RelationRow>,
    goal_tags: Vec<RelationRow>,
    goal_instance_tags: Vec<RelationRow>,
    task_tags: Vec<RelationRow>,
}

impl LegacyData {
    fn counts(&self) -> LegacyImportCounts {
        LegacyImportCounts {
            entries: self.entries.len() as u64,
            tags: self.tags.len() as u64,
            goals: self.goals.len() as u64,
            goal_instances: self.goal_instances.len() as u64,
            tasks: self.tasks.len() as u64,
            subtasks: self.subtasks.len() as u64,
            entry_tags: self.entry_tags.len() as u64,
            goal_tags: self.goal_tags.len() as u64,
            goal_instance_tags: self.goal_instance_tags.len() as u64,
            task_tags: self.task_tags.len() as u64,
        }
    }
}

struct EntryRow {
    id: String,
    document: String,
    created_at: String,
    is_pinned: i64,
    is_archived: i64,
    is_deleted: i64,
    updated_at: String,
    deleted_at: Option<String>,
}

struct TagRow {
    id: String,
    name: String,
    created_at: String,
    updated_at: String,
    deleted_at: Option<String>,
}

struct GoalRow {
    id: String,
    name: String,
    description: Option<String>,
    is_non_recurring: i64,
    recurrence_type: Option<String>,
    recurrence_interval: Option<i64>,
    recurrence_anchor: Option<String>,
    recurrence_meta: Option<String>,
    timezone: String,
    created_at: String,
    updated_at: String,
    deleted_at: Option<String>,
}

struct GoalInstanceRow {
    id: String,
    goal_id: String,
    period_start: String,
    period_end: Option<String>,
    status: String,
    created_at: String,
}

struct TaskRow {
    id: String,
    title: String,
    description: Option<String>,
    is_completed: i64,
    due_date: Option<String>,
    goal_instance_id: Option<String>,
    goal_id: Option<String>,
    created_at: String,
    updated_at: String,
    deleted_at: Option<String>,
}

struct SubtaskRow {
    id: String,
    title: String,
    is_completed: i64,
    task_id: String,
    order_index: i64,
    created_at: String,
    updated_at: String,
    deleted_at: Option<String>,
}

struct RelationRow {
    parent_id: String,
    tag_id: String,
}

pub async fn preview_legacy_database(source_path: &str) -> Result<LegacyImportPreview> {
    let data = read_legacy_database(source_path).await?;
    Ok(LegacyImportPreview {
        source_path: source_path.to_string(),
        counts: data.counts(),
    })
}

pub async fn import_legacy_database(
    database: &Database,
    source_path: &str,
) -> Result<LegacyImportCounts> {
    let data = read_legacy_database(source_path).await?;
    let conn = database.connect()?;
    conn.execute("BEGIN IMMEDIATE", libsql::params![]).await?;

    let result = write_legacy_data(&conn, data).await;
    match result {
        Ok(counts) => {
            conn.execute("COMMIT", libsql::params![]).await?;
            Ok(counts)
        }
        Err(error) => {
            let _ = conn.execute("ROLLBACK", libsql::params![]).await;
            Err(error)
        }
    }
}

async fn read_legacy_database(source_path: &str) -> Result<LegacyData> {
    let path = Path::new(source_path);
    if !path.is_file() {
        return Err(AppError::BadRequest(format!(
            "Legacy database not found: {}",
            path.display()
        )));
    }

    let source = Builder::new_local(path)
        .flags(OpenFlags::SQLITE_OPEN_READ_ONLY)
        .build()
        .await?;
    let conn = source.connect()?;
    verify_legacy_schema(&conn).await?;

    Ok(LegacyData {
        entries: read_entries(&conn).await?,
        tags: read_tags(&conn).await?,
        goals: read_goals(&conn).await?,
        goal_instances: read_goal_instances(&conn).await?,
        tasks: read_tasks(&conn).await?,
        subtasks: read_subtasks(&conn).await?,
        entry_tags: read_relations(&conn, "entry_tags", "entry_id").await?,
        goal_tags: read_relations(&conn, "goal_tags", "goal_id").await?,
        goal_instance_tags: read_relations(&conn, "goal_instance_tags", "goal_instance_id").await?,
        task_tags: read_relations(&conn, "task_tags", "task_id").await?,
    })
}

async fn verify_legacy_schema(conn: &Connection) -> Result<()> {
    let mut rows = conn
        .query(
            "SELECT name FROM sqlite_master WHERE type = 'table'",
            libsql::params![],
        )
        .await?;
    let mut tables = HashSet::new();
    while let Some(row) = rows.next().await? {
        tables.insert(row.get::<String>(0)?);
    }

    let missing: Vec<_> = REQUIRED_TABLES
        .iter()
        .filter(|table| !tables.contains(**table))
        .copied()
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "This is not a supported January 2026 Aether database; missing: {}",
            missing.join(", ")
        )))
    }
}

async fn read_entries(conn: &Connection) -> Result<Vec<EntryRow>> {
    let mut rows = conn.query("SELECT id, document, created_at, is_pinned, is_archived, is_deleted, updated_at, deleted_at FROM entries", libsql::params![]).await?;
    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        values.push(EntryRow {
            id: row.get(0)?,
            document: row.get(1)?,
            created_at: row.get(2)?,
            is_pinned: row.get(3)?,
            is_archived: row.get(4)?,
            is_deleted: row.get(5)?,
            updated_at: row.get(6)?,
            deleted_at: row.get(7)?,
        });
    }
    Ok(values)
}

async fn read_tags(conn: &Connection) -> Result<Vec<TagRow>> {
    let mut rows = conn
        .query(
            "SELECT id, name, created_at, updated_at, deleted_at FROM tags",
            libsql::params![],
        )
        .await?;
    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        values.push(TagRow {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            deleted_at: row.get(4)?,
        });
    }
    Ok(values)
}

async fn read_goals(conn: &Connection) -> Result<Vec<GoalRow>> {
    let mut rows = conn.query("SELECT id, name, description, is_non_recurring, recurrence_type, recurrence_interval, recurrence_anchor, recurrence_meta, timezone, created_at, updated_at, deleted_at FROM goals", libsql::params![]).await?;
    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        values.push(GoalRow {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            is_non_recurring: row.get(3)?,
            recurrence_type: row.get(4)?,
            recurrence_interval: row.get(5)?,
            recurrence_anchor: row.get(6)?,
            recurrence_meta: row.get(7)?,
            timezone: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
            deleted_at: row.get(11)?,
        });
    }
    Ok(values)
}

async fn read_goal_instances(conn: &Connection) -> Result<Vec<GoalInstanceRow>> {
    let mut rows = conn
        .query(
            "SELECT id, goal_id, period_start, period_end, status, created_at FROM goal_instances",
            libsql::params![],
        )
        .await?;
    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        values.push(GoalInstanceRow {
            id: row.get(0)?,
            goal_id: row.get(1)?,
            period_start: row.get(2)?,
            period_end: row.get(3)?,
            status: row.get(4)?,
            created_at: row.get(5)?,
        });
    }
    Ok(values)
}

async fn read_tasks(conn: &Connection) -> Result<Vec<TaskRow>> {
    let mut rows = conn.query("SELECT id, title, description, is_completed, due_date, goal_instance_id, goal_id, created_at, updated_at, deleted_at FROM tasks", libsql::params![]).await?;
    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        values.push(TaskRow {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            is_completed: row.get(3)?,
            due_date: row.get(4)?,
            goal_instance_id: row.get(5)?,
            goal_id: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            deleted_at: row.get(9)?,
        });
    }
    Ok(values)
}

async fn read_subtasks(conn: &Connection) -> Result<Vec<SubtaskRow>> {
    let mut rows = conn.query("SELECT id, title, is_completed, task_id, order_index, created_at, updated_at, deleted_at FROM sub_tasks", libsql::params![]).await?;
    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        values.push(SubtaskRow {
            id: row.get(0)?,
            title: row.get(1)?,
            is_completed: row.get(2)?,
            task_id: row.get(3)?,
            order_index: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
            deleted_at: row.get(7)?,
        });
    }
    Ok(values)
}

async fn read_relations(
    conn: &Connection,
    table: &str,
    parent_column: &str,
) -> Result<Vec<RelationRow>> {
    let query = format!("SELECT {parent_column}, tag_id FROM {table}");
    let mut rows = conn.query(&query, libsql::params![]).await?;
    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        values.push(RelationRow {
            parent_id: row.get(0)?,
            tag_id: row.get(1)?,
        });
    }
    Ok(values)
}

async fn write_legacy_data(conn: &Connection, data: LegacyData) -> Result<LegacyImportCounts> {
    let mut counts = LegacyImportCounts::default();
    let imported_at = Utc::now().timestamp_millis();

    for row in data.tags {
        let (created_at, _) = normalize_timestamp(&row.created_at, "tag created_at")?;
        let (updated_at, updated_ms) = normalize_timestamp(&row.updated_at, "tag updated_at")?;
        let deleted_at = normalize_optional_timestamp(row.deleted_at, "tag deleted_at")?;
        counts.tags += conn.execute("INSERT OR IGNORE INTO tags (id, name, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, '{}')", libsql::params![row.id.clone(), row.name, created_at, updated_at, deleted_at.clone(), row.id, updated_ms, i64::from(deleted_at.is_some())]).await?;
    }

    for row in data.entries {
        let (created_at, _) = normalize_timestamp(&row.created_at, "entry created_at")?;
        let (updated_at, updated_ms) = normalize_timestamp(&row.updated_at, "entry updated_at")?;
        let deleted_at = normalize_optional_timestamp(row.deleted_at, "entry deleted_at")?;
        let is_deleted = i64::from(row.is_deleted != 0 || deleted_at.is_some());
        counts.entries += conn.execute("INSERT OR IGNORE INTO entries (id, document, created_at, is_pinned, is_archived, is_deleted, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, '{}')", libsql::params![row.id.clone(), row.document, created_at, row.is_pinned, row.is_archived, row.is_deleted, updated_at, deleted_at, row.id, updated_ms, is_deleted]).await?;
    }

    for row in data.goals {
        let (created_at, _) = normalize_timestamp(&row.created_at, "goal created_at")?;
        let (updated_at, updated_ms) = normalize_timestamp(&row.updated_at, "goal updated_at")?;
        let recurrence_anchor =
            normalize_optional_timestamp(row.recurrence_anchor, "goal recurrence_anchor")?;
        let deleted_at = normalize_optional_timestamp(row.deleted_at, "goal deleted_at")?;
        let is_deleted = i64::from(deleted_at.is_some());
        counts.goals += conn.execute("INSERT OR IGNORE INTO goals (id, name, description, is_non_recurring, recurrence_type, recurrence_interval, recurrence_anchor, recurrence_meta, timezone, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, '{}')", libsql::params![row.id.clone(), row.name, row.description, row.is_non_recurring, row.recurrence_type, row.recurrence_interval, recurrence_anchor, row.recurrence_meta, row.timezone, created_at, updated_at, deleted_at, row.id, updated_ms, is_deleted]).await?;
    }

    for row in data.goal_instances {
        let (period_start, _) =
            normalize_timestamp(&row.period_start, "goal instance period_start")?;
        let period_end = normalize_optional_timestamp(row.period_end, "goal instance period_end")?;
        let (created_at, created_ms) =
            normalize_timestamp(&row.created_at, "goal instance created_at")?;
        counts.goal_instances += conn.execute("INSERT OR IGNORE INTO goal_instances (id, goal_id, period_start, period_end, status, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, NULL, ?7, ?8, 0, '{}')", libsql::params![row.id.clone(), row.goal_id, period_start, period_end, row.status, created_at, row.id, created_ms]).await?;
    }

    for row in data.tasks {
        let (created_at, _) = normalize_timestamp(&row.created_at, "task created_at")?;
        let (updated_at, updated_ms) = normalize_timestamp(&row.updated_at, "task updated_at")?;
        let due_date = normalize_optional_timestamp(row.due_date, "task due_date")?;
        let deleted_at = normalize_optional_timestamp(row.deleted_at, "task deleted_at")?;
        let is_deleted = i64::from(deleted_at.is_some());
        counts.tasks += conn.execute("INSERT OR IGNORE INTO tasks (id, title, description, is_completed, due_date, goal_instance_id, goal_id, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, '{}')", libsql::params![row.id.clone(), row.title, row.description, row.is_completed, due_date, row.goal_instance_id, row.goal_id, created_at, updated_at, deleted_at, row.id, updated_ms, is_deleted]).await?;
    }

    for row in data.subtasks {
        let (created_at, _) = normalize_timestamp(&row.created_at, "subtask created_at")?;
        let (updated_at, updated_ms) = normalize_timestamp(&row.updated_at, "subtask updated_at")?;
        let deleted_at = normalize_optional_timestamp(row.deleted_at, "subtask deleted_at")?;
        let is_deleted = i64::from(deleted_at.is_some());
        counts.subtasks += conn.execute("INSERT OR IGNORE INTO subtasks (id, title, is_completed, task_id, order_index, created_at, updated_at, deleted_at, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, '{}')", libsql::params![row.id.clone(), row.title, row.is_completed, row.task_id, row.order_index, created_at, updated_at, deleted_at, row.id, updated_ms, is_deleted]).await?;
    }

    counts.entry_tags +=
        write_relations(conn, "entry_tags", "entry_id", data.entry_tags, imported_at).await?;
    counts.goal_tags +=
        write_relations(conn, "goal_tags", "goal_id", data.goal_tags, imported_at).await?;
    counts.goal_instance_tags += write_relations(
        conn,
        "goal_instance_tags",
        "goal_instance_id",
        data.goal_instance_tags,
        imported_at,
    )
    .await?;
    counts.task_tags +=
        write_relations(conn, "task_tags", "task_id", data.task_tags, imported_at).await?;
    Ok(counts)
}

async fn write_relations(
    conn: &Connection,
    table: &str,
    parent_column: &str,
    rows: Vec<RelationRow>,
    updated_at: i64,
) -> Result<u64> {
    let query = format!("INSERT OR IGNORE INTO {table} ({parent_column}, tag_id, _sync_id, _updated_at, _deleted, _extra) VALUES (?1, ?2, ?3, ?4, 0, '{{}}')");
    let mut inserted = 0;
    for row in rows {
        let sync_id = format!("{}|{}", row.parent_id, row.tag_id);
        inserted += conn
            .execute(
                &query,
                libsql::params![row.parent_id, row.tag_id, sync_id, updated_at],
            )
            .await?;
    }
    Ok(inserted)
}

fn normalize_optional_timestamp(value: Option<String>, context: &str) -> Result<Option<String>> {
    value
        .map(|value| normalize_timestamp(&value, context).map(|(value, _)| value))
        .transpose()
}

fn normalize_timestamp(value: &str, context: &str) -> Result<(String, i64)> {
    let timestamp = DateTime::parse_from_rfc3339(value)
        .or_else(|_| DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f%:z"))
        .map_err(|_| {
            AppError::BadRequest(format!(
                "Unsupported legacy timestamp for {context}: {value}"
            ))
        })?
        .with_timezone(&Utc);
    Ok((
        timestamp.to_rfc3339_opts(SecondsFormat::Millis, true),
        timestamp.timestamp_millis(),
    ))
}
