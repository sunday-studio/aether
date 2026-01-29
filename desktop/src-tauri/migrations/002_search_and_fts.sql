-- Full-text search infrastructure with correct settings from the start
-- FTS5 indexes with detail='full' to support phrase queries

-- Entries FTS index
CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
    document,
    tokenize='trigram',
    detail='full'
);

-- Tasks FTS index (title + description)
CREATE VIRTUAL TABLE IF NOT EXISTS tasks_fts USING fts5(
    title,
    description,
    tokenize='trigram',
    detail='full'
);

-- Subtasks FTS index
CREATE VIRTUAL TABLE IF NOT EXISTS subtasks_fts USING fts5(
    title,
    tokenize='trigram',
    detail='full'
);

-- Goals FTS index (name + description)
CREATE VIRTUAL TABLE IF NOT EXISTS goals_fts USING fts5(
    name,
    description,
    tokenize='trigram',
    detail='full'
);

-- Tags FTS index
CREATE VIRTUAL TABLE IF NOT EXISTS tags_fts USING fts5(
    name,
    tokenize='trigram',
    detail='full'
);

-- Bookmarks FTS index
CREATE VIRTUAL TABLE IF NOT EXISTS bookmarks_fts USING fts5(
    title,
    description,
    site_name,
    author,
    tokenize='trigram',
    detail='full'
);

-- Mapping tables to convert TEXT ids to INTEGER rowids for FTS5

CREATE TABLE IF NOT EXISTS entries_fts_map (
    rowid INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id TEXT NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_entries_fts_map_entry_id ON entries_fts_map(entry_id);

CREATE TABLE IF NOT EXISTS tasks_fts_map (
    rowid INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_tasks_fts_map_task_id ON tasks_fts_map(task_id);

CREATE TABLE IF NOT EXISTS subtasks_fts_map (
    rowid INTEGER PRIMARY KEY AUTOINCREMENT,
    subtask_id TEXT NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_subtasks_fts_map_subtask_id ON subtasks_fts_map(subtask_id);

CREATE TABLE IF NOT EXISTS goals_fts_map (
    rowid INTEGER PRIMARY KEY AUTOINCREMENT,
    goal_id TEXT NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_goals_fts_map_goal_id ON goals_fts_map(goal_id);

CREATE TABLE IF NOT EXISTS tags_fts_map (
    rowid INTEGER PRIMARY KEY AUTOINCREMENT,
    tag_id TEXT NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_tags_fts_map_tag_id ON tags_fts_map(tag_id);

CREATE TABLE IF NOT EXISTS bookmarks_fts_map (
    rowid INTEGER PRIMARY KEY AUTOINCREMENT,
    bookmark_id TEXT NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_bookmarks_fts_map_bookmark_id ON bookmarks_fts_map(bookmark_id);
