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

-- Triggers to keep FTS indexes in sync with base tables

-- Entries triggers
CREATE TRIGGER IF NOT EXISTS entries_fts_insert AFTER INSERT ON entries BEGIN
    INSERT INTO entries_fts_map(entry_id) VALUES (new.id);
    INSERT INTO entries_fts(rowid, document) 
    VALUES ((SELECT rowid FROM entries_fts_map WHERE entry_id = new.id), new.document);
END;

CREATE TRIGGER IF NOT EXISTS entries_fts_delete AFTER DELETE ON entries BEGIN
    DELETE FROM entries_fts WHERE rowid = (SELECT rowid FROM entries_fts_map WHERE entry_id = old.id);
    DELETE FROM entries_fts_map WHERE entry_id = old.id;
END;

CREATE TRIGGER IF NOT EXISTS entries_fts_update AFTER UPDATE ON entries BEGIN
    DELETE FROM entries_fts WHERE rowid = (SELECT rowid FROM entries_fts_map WHERE entry_id = old.id);
    INSERT INTO entries_fts(rowid, document) 
    VALUES ((SELECT rowid FROM entries_fts_map WHERE entry_id = new.id), new.document);
END;

-- Tasks triggers
CREATE TRIGGER IF NOT EXISTS tasks_fts_insert AFTER INSERT ON tasks BEGIN
    INSERT INTO tasks_fts_map(task_id) VALUES (new.id);
    INSERT INTO tasks_fts(rowid, title, description) 
    VALUES ((SELECT rowid FROM tasks_fts_map WHERE task_id = new.id), new.title, COALESCE(new.description, ''));
END;

CREATE TRIGGER IF NOT EXISTS tasks_fts_delete AFTER DELETE ON tasks BEGIN
    DELETE FROM tasks_fts WHERE rowid = (SELECT rowid FROM tasks_fts_map WHERE task_id = old.id);
    DELETE FROM tasks_fts_map WHERE task_id = old.id;
END;

CREATE TRIGGER IF NOT EXISTS tasks_fts_update AFTER UPDATE ON tasks BEGIN
    DELETE FROM tasks_fts WHERE rowid = (SELECT rowid FROM tasks_fts_map WHERE task_id = old.id);
    INSERT INTO tasks_fts(rowid, title, description) 
    VALUES ((SELECT rowid FROM tasks_fts_map WHERE task_id = new.id), new.title, COALESCE(new.description, ''));
END;

-- Subtasks triggers
CREATE TRIGGER IF NOT EXISTS subtasks_fts_insert AFTER INSERT ON subtasks BEGIN
    INSERT INTO subtasks_fts_map(subtask_id) VALUES (new.id);
    INSERT INTO subtasks_fts(rowid, title) 
    VALUES ((SELECT rowid FROM subtasks_fts_map WHERE subtask_id = new.id), new.title);
END;

CREATE TRIGGER IF NOT EXISTS subtasks_fts_delete AFTER DELETE ON subtasks BEGIN
    DELETE FROM subtasks_fts WHERE rowid = (SELECT rowid FROM subtasks_fts_map WHERE subtask_id = old.id);
    DELETE FROM subtasks_fts_map WHERE subtask_id = old.id;
END;

CREATE TRIGGER IF NOT EXISTS subtasks_fts_update AFTER UPDATE ON subtasks BEGIN
    DELETE FROM subtasks_fts WHERE rowid = (SELECT rowid FROM subtasks_fts_map WHERE subtask_id = old.id);
    INSERT INTO subtasks_fts(rowid, title) 
    VALUES ((SELECT rowid FROM subtasks_fts_map WHERE subtask_id = new.id), new.title);
END;

-- Goals triggers
CREATE TRIGGER IF NOT EXISTS goals_fts_insert AFTER INSERT ON goals BEGIN
    INSERT INTO goals_fts_map(goal_id) VALUES (new.id);
    INSERT INTO goals_fts(rowid, name, description) 
    VALUES ((SELECT rowid FROM goals_fts_map WHERE goal_id = new.id), new.name, COALESCE(new.description, ''));
END;

CREATE TRIGGER IF NOT EXISTS goals_fts_delete AFTER DELETE ON goals BEGIN
    DELETE FROM goals_fts WHERE rowid = (SELECT rowid FROM goals_fts_map WHERE goal_id = old.id);
    DELETE FROM goals_fts_map WHERE goal_id = old.id;
END;

CREATE TRIGGER IF NOT EXISTS goals_fts_update AFTER UPDATE ON goals BEGIN
    DELETE FROM goals_fts WHERE rowid = (SELECT rowid FROM goals_fts_map WHERE goal_id = old.id);
    INSERT INTO goals_fts(rowid, name, description) 
    VALUES ((SELECT rowid FROM goals_fts_map WHERE goal_id = new.id), new.name, COALESCE(new.description, ''));
END;

-- Tags triggers
CREATE TRIGGER IF NOT EXISTS tags_fts_insert AFTER INSERT ON tags BEGIN
    INSERT INTO tags_fts_map(tag_id) VALUES (new.id);
    INSERT INTO tags_fts(rowid, name) 
    VALUES ((SELECT rowid FROM tags_fts_map WHERE tag_id = new.id), new.name);
END;

CREATE TRIGGER IF NOT EXISTS tags_fts_delete AFTER DELETE ON tags BEGIN
    DELETE FROM tags_fts WHERE rowid = (SELECT rowid FROM tags_fts_map WHERE tag_id = old.id);
    DELETE FROM tags_fts_map WHERE tag_id = old.id;
END;

CREATE TRIGGER IF NOT EXISTS tags_fts_update AFTER UPDATE ON tags BEGIN
    DELETE FROM tags_fts WHERE rowid = (SELECT rowid FROM tags_fts_map WHERE tag_id = old.id);
    INSERT INTO tags_fts(rowid, name) 
    VALUES ((SELECT rowid FROM tags_fts_map WHERE tag_id = new.id), new.name);
END;

-- Bookmarks triggers
CREATE TRIGGER IF NOT EXISTS bookmarks_fts_insert AFTER INSERT ON bookmarks BEGIN
    INSERT INTO bookmarks_fts_map(bookmark_id) VALUES (new.id);
    INSERT INTO bookmarks_fts(rowid, title, description, site_name, author)
    VALUES ((SELECT rowid FROM bookmarks_fts_map WHERE bookmark_id = new.id), 
            COALESCE(new.title, ''), COALESCE(new.description, ''), 
            COALESCE(new.site_name, ''), COALESCE(new.author, ''));
END;

CREATE TRIGGER IF NOT EXISTS bookmarks_fts_delete AFTER DELETE ON bookmarks BEGIN
    DELETE FROM bookmarks_fts WHERE rowid = (SELECT rowid FROM bookmarks_fts_map WHERE bookmark_id = old.id);
    DELETE FROM bookmarks_fts_map WHERE bookmark_id = old.id;
END;

CREATE TRIGGER IF NOT EXISTS bookmarks_fts_update AFTER UPDATE ON bookmarks BEGIN
    DELETE FROM bookmarks_fts WHERE rowid = (SELECT rowid FROM bookmarks_fts_map WHERE bookmark_id = old.id);
    INSERT INTO bookmarks_fts(rowid, title, description, site_name, author)
    VALUES ((SELECT rowid FROM bookmarks_fts_map WHERE bookmark_id = new.id), 
            COALESCE(new.title, ''), COALESCE(new.description, ''), 
            COALESCE(new.site_name, ''), COALESCE(new.author, ''));
END;

-- Backfill existing data into FTS indexes
INSERT INTO entries_fts_map(entry_id) 
SELECT id FROM entries WHERE deleted_at IS NULL;

INSERT INTO entries_fts(rowid, document) 
SELECT m.rowid, e.document 
FROM entries e 
JOIN entries_fts_map m ON e.id = m.entry_id 
WHERE e.deleted_at IS NULL;

INSERT INTO tasks_fts_map(task_id) 
SELECT id FROM tasks WHERE deleted_at IS NULL;

INSERT INTO tasks_fts(rowid, title, description) 
SELECT m.rowid, t.title, COALESCE(t.description, '') 
FROM tasks t 
JOIN tasks_fts_map m ON t.id = m.task_id 
WHERE t.deleted_at IS NULL;

INSERT INTO subtasks_fts_map(subtask_id) 
SELECT id FROM subtasks WHERE deleted_at IS NULL;

INSERT INTO subtasks_fts(rowid, title) 
SELECT m.rowid, s.title 
FROM subtasks s 
JOIN subtasks_fts_map m ON s.id = m.subtask_id 
WHERE s.deleted_at IS NULL;

INSERT INTO goals_fts_map(goal_id) 
SELECT id FROM goals WHERE deleted_at IS NULL;

INSERT INTO goals_fts(rowid, name, description) 
SELECT m.rowid, g.name, COALESCE(g.description, '') 
FROM goals g 
JOIN goals_fts_map m ON g.id = m.goal_id 
WHERE g.deleted_at IS NULL;

INSERT INTO tags_fts_map(tag_id) 
SELECT id FROM tags WHERE deleted_at IS NULL;

INSERT INTO tags_fts(rowid, name) 
SELECT m.rowid, t.name 
FROM tags t 
JOIN tags_fts_map m ON t.id = m.tag_id 
WHERE t.deleted_at IS NULL;

INSERT INTO bookmarks_fts_map(bookmark_id) 
SELECT id FROM bookmarks WHERE deleted_at IS NULL;

INSERT INTO bookmarks_fts(rowid, title, description, site_name, author)
SELECT m.rowid, COALESCE(b.title, ''), COALESCE(b.description, ''), 
       COALESCE(b.site_name, ''), COALESCE(b.author, '')
FROM bookmarks b
JOIN bookmarks_fts_map m ON b.id = m.bookmark_id
WHERE b.deleted_at IS NULL;
