use super::{legacy_import, migrations};
use libsql::Builder;
use std::fs;

#[tokio::test]
async fn imports_january_legacy_database_and_normalizes_timestamps() {
    let root = std::env::temp_dir().join(format!("aether-legacy-import-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let source_path = root.join("legacy.db");
    let target_path = root.join("current.db");

    let source = Builder::new_local(&source_path).build().await.unwrap();
    let source_conn = source.connect().unwrap();
    for statement in [
        "CREATE TABLE entries (id TEXT PRIMARY KEY, document TEXT NOT NULL, created_at TEXT NOT NULL, is_pinned INTEGER, is_archived INTEGER, is_deleted INTEGER, updated_at TEXT NOT NULL, deleted_at TEXT)",
        "CREATE TABLE tags (id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT)",
        "CREATE TABLE goals (id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT, is_non_recurring INTEGER, recurrence_type TEXT, recurrence_interval INTEGER, recurrence_anchor TEXT, recurrence_meta TEXT, timezone TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT)",
        "CREATE TABLE goal_instances (id TEXT PRIMARY KEY, goal_id TEXT NOT NULL, period_start TEXT NOT NULL, period_end TEXT, status TEXT NOT NULL, created_at TEXT NOT NULL)",
        "CREATE TABLE tasks (id TEXT PRIMARY KEY, title TEXT NOT NULL, description TEXT, is_completed INTEGER, due_date TEXT, goal_instance_id TEXT, goal_id TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT)",
        "CREATE TABLE sub_tasks (id TEXT PRIMARY KEY, title TEXT NOT NULL, is_completed INTEGER, task_id TEXT NOT NULL, order_index INTEGER, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT)",
        "CREATE TABLE entry_tags (entry_id TEXT NOT NULL, tag_id TEXT NOT NULL)",
        "CREATE TABLE goal_tags (goal_id TEXT NOT NULL, tag_id TEXT NOT NULL)",
        "CREATE TABLE goal_instance_tags (goal_instance_id TEXT NOT NULL, tag_id TEXT NOT NULL)",
        "CREATE TABLE task_tags (task_id TEXT NOT NULL, tag_id TEXT NOT NULL)",
    ] {
        source_conn.execute(statement, libsql::params![]).await.unwrap();
    }

    source_conn.execute("INSERT INTO tags VALUES ('tag_1', 'Home', '2026-01-01T10:00:00Z', '2026-01-01T10:00:00Z', NULL)", libsql::params![]).await.unwrap();
    source_conn.execute("INSERT INTO entries VALUES ('entry_1', 'Legacy entry', '2026-01-01T10:00:00Z', 0, 0, 0, '2026-01-02 11:00:00.000+01:00', NULL)", libsql::params![]).await.unwrap();
    source_conn.execute("INSERT INTO goals VALUES ('goal_1', 'Legacy goal', NULL, 0, 'weekly', 1, '2026-01-01 10:00:00.000+00:00', '{}', 'Europe/Amsterdam', '2026-01-01T10:00:00Z', '2026-01-01T10:00:00Z', NULL)", libsql::params![]).await.unwrap();
    source_conn.execute("INSERT INTO goal_instances VALUES ('goal-instance_1', 'goal_1', '2026-01-01 10:00:00.000+00:00', '2026-01-08 10:00:00.000+00:00', 'active', '2026-01-01T10:00:00Z')", libsql::params![]).await.unwrap();
    source_conn.execute("INSERT INTO tasks VALUES ('task_1', 'Legacy task', NULL, 0, NULL, 'goal-instance_1', 'goal_1', '2026-01-01T10:00:00Z', '2026-01-01T10:00:00Z', NULL)", libsql::params![]).await.unwrap();
    source_conn.execute("INSERT INTO sub_tasks VALUES ('subtask_1', 'Legacy subtask', 0, 'task_1', 0, '2026-01-01T10:00:00Z', '2026-01-01T10:00:00Z', NULL)", libsql::params![]).await.unwrap();
    source_conn
        .execute(
            "INSERT INTO entry_tags VALUES ('entry_1', 'tag_1')",
            libsql::params![],
        )
        .await
        .unwrap();
    source_conn
        .execute(
            "INSERT INTO goal_tags VALUES ('goal_1', 'tag_1')",
            libsql::params![],
        )
        .await
        .unwrap();
    source_conn
        .execute(
            "INSERT INTO goal_instance_tags VALUES ('goal-instance_1', 'tag_1')",
            libsql::params![],
        )
        .await
        .unwrap();
    source_conn
        .execute(
            "INSERT INTO task_tags VALUES ('task_1', 'tag_1')",
            libsql::params![],
        )
        .await
        .unwrap();
    drop(source_conn);
    drop(source);

    let preview = legacy_import::preview_legacy_database(source_path.to_str().unwrap())
        .await
        .unwrap();
    assert_eq!(preview.counts.entries, 1);
    assert_eq!(preview.counts.subtasks, 1);

    let target = Builder::new_local(&target_path).build().await.unwrap();
    migrations::run_migrations(&target).await.unwrap();
    let imported = legacy_import::import_legacy_database(&target, source_path.to_str().unwrap())
        .await
        .unwrap();
    assert_eq!(imported.entries, 1);
    assert_eq!(imported.goal_instances, 1);
    assert_eq!(imported.task_tags, 1);

    let target_conn = target.connect().unwrap();
    let mut rows = target_conn
        .query(
            "SELECT created_at, updated_at FROM entries WHERE id = 'entry_1'",
            libsql::params![],
        )
        .await
        .unwrap();
    let entry = rows.next().await.unwrap().unwrap();
    let created_at: String = entry.get(0).unwrap();
    let updated_at: String = entry.get(1).unwrap();
    assert_eq!(created_at, "2026-01-01T10:00:00.000Z");
    assert_eq!(updated_at, "2026-01-02T10:00:00.000Z");
    drop(entry);
    drop(rows);
    drop(target_conn);
    assert_eq!(
        legacy_import::import_legacy_database(&target, source_path.to_str().unwrap())
            .await
            .unwrap()
            .entries,
        0
    );
    let _ = fs::remove_dir_all(root);
}
