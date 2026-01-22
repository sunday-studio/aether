-- Migration: Fix FTS5 detail setting to support phrase queries
-- Version: 011
-- Description: Recreates all FTS5 tables with detail='full' instead of detail='column' to support phrase queries

-- Drop all existing FTS5 triggers first
DROP TRIGGER IF EXISTS entries_fts_insert;
DROP TRIGGER IF EXISTS entries_fts_delete;
DROP TRIGGER IF EXISTS entries_fts_update;
DROP TRIGGER IF EXISTS tasks_fts_insert;
DROP TRIGGER IF EXISTS tasks_fts_delete;
DROP TRIGGER IF EXISTS tasks_fts_update;
DROP TRIGGER IF EXISTS subtasks_fts_insert;
DROP TRIGGER IF EXISTS subtasks_fts_delete;
DROP TRIGGER IF EXISTS subtasks_fts_update;
DROP TRIGGER IF EXISTS goals_fts_insert;
DROP TRIGGER IF EXISTS goals_fts_delete;
DROP TRIGGER IF EXISTS goals_fts_update;
DROP TRIGGER IF EXISTS tags_fts_insert;
DROP TRIGGER IF EXISTS tags_fts_delete;
DROP TRIGGER IF EXISTS tags_fts_update;
DROP TRIGGER IF EXISTS bookmarks_fts_insert;
DROP TRIGGER IF EXISTS bookmarks_fts_delete;
DROP TRIGGER IF EXISTS bookmarks_fts_update;

-- Drop all existing FTS5 tables
DROP TABLE IF EXISTS entries_fts;
DROP TABLE IF EXISTS tasks_fts;
DROP TABLE IF EXISTS subtasks_fts;
DROP TABLE IF EXISTS goals_fts;
DROP TABLE IF EXISTS tags_fts;
DROP TABLE IF EXISTS bookmarks_fts;

-- Recreate FTS5 tables with detail='full' to support phrase queries
CREATE VIRTUAL TABLE entries_fts USING fts5(
    document,
    tokenize='trigram',
    detail='full'
);

CREATE VIRTUAL TABLE tasks_fts USING fts5(
    title,
    description,
    tokenize='trigram',
    detail='full'
);

CREATE VIRTUAL TABLE subtasks_fts USING fts5(
    title,
    tokenize='trigram',
    detail='full'
);

CREATE VIRTUAL TABLE goals_fts USING fts5(
    name,
    description,
    tokenize='trigram',
    detail='full'
);

CREATE VIRTUAL TABLE tags_fts USING fts5(
    name,
    tokenize='trigram',
    detail='full'
);

CREATE VIRTUAL TABLE bookmarks_fts USING fts5(
    title,
    description,
    site_name,
    author,
    tokenize='trigram',
    detail='full'
);

-- Recreate all triggers

-- Entries triggers
CREATE TRIGGER entries_fts_insert AFTER INSERT ON entries BEGIN
    INSERT INTO entries_fts_map(entry_id) VALUES (new.id);
    INSERT INTO entries_fts(rowid, document) 
    VALUES ((SELECT rowid FROM entries_fts_map WHERE entry_id = new.id), new.document);
END;

CREATE TRIGGER entries_fts_delete AFTER DELETE ON entries BEGIN
    DELETE FROM entries_fts WHERE rowid = (SELECT rowid FROM entries_fts_map WHERE entry_id = old.id);
    DELETE FROM entries_fts_map WHERE entry_id = old.id;
END;

CREATE TRIGGER entries_fts_update AFTER UPDATE ON entries BEGIN
    DELETE FROM entries_fts WHERE rowid = (SELECT rowid FROM entries_fts_map WHERE entry_id = old.id);
    INSERT INTO entries_fts(rowid, document) 
    VALUES ((SELECT rowid FROM entries_fts_map WHERE entry_id = new.id), new.document);
END;

-- Tasks triggers
CREATE TRIGGER tasks_fts_insert AFTER INSERT ON tasks BEGIN
    INSERT INTO tasks_fts_map(task_id) VALUES (new.id);
    INSERT INTO tasks_fts(rowid, title, description) 
    VALUES ((SELECT rowid FROM tasks_fts_map WHERE task_id = new.id), new.title, COALESCE(new.description, ''));
END;

CREATE TRIGGER tasks_fts_delete AFTER DELETE ON tasks BEGIN
    DELETE FROM tasks_fts WHERE rowid = (SELECT rowid FROM tasks_fts_map WHERE task_id = old.id);
    DELETE FROM tasks_fts_map WHERE task_id = old.id;
END;

CREATE TRIGGER tasks_fts_update AFTER UPDATE ON tasks BEGIN
    DELETE FROM tasks_fts WHERE rowid = (SELECT rowid FROM tasks_fts_map WHERE task_id = old.id);
    INSERT INTO tasks_fts(rowid, title, description) 
    VALUES ((SELECT rowid FROM tasks_fts_map WHERE task_id = new.id), new.title, COALESCE(new.description, ''));
END;

-- Subtasks triggers
CREATE TRIGGER subtasks_fts_insert AFTER INSERT ON subtasks BEGIN
    INSERT INTO subtasks_fts_map(subtask_id) VALUES (new.id);
    INSERT INTO subtasks_fts(rowid, title) 
    VALUES ((SELECT rowid FROM subtasks_fts_map WHERE subtask_id = new.id), new.title);
END;

CREATE TRIGGER subtasks_fts_delete AFTER DELETE ON subtasks BEGIN
    DELETE FROM subtasks_fts WHERE rowid = (SELECT rowid FROM subtasks_fts_map WHERE subtask_id = old.id);
    DELETE FROM subtasks_fts_map WHERE subtask_id = old.id;
END;

CREATE TRIGGER subtasks_fts_update AFTER UPDATE ON subtasks BEGIN
    DELETE FROM subtasks_fts WHERE rowid = (SELECT rowid FROM subtasks_fts_map WHERE subtask_id = old.id);
    INSERT INTO subtasks_fts(rowid, title) 
    VALUES ((SELECT rowid FROM subtasks_fts_map WHERE subtask_id = new.id), new.title);
END;

-- Goals triggers
CREATE TRIGGER goals_fts_insert AFTER INSERT ON goals BEGIN
    INSERT INTO goals_fts_map(goal_id) VALUES (new.id);
    INSERT INTO goals_fts(rowid, name, description) 
    VALUES ((SELECT rowid FROM goals_fts_map WHERE goal_id = new.id), new.name, COALESCE(new.description, ''));
END;

CREATE TRIGGER goals_fts_delete AFTER DELETE ON goals BEGIN
    DELETE FROM goals_fts WHERE rowid = (SELECT rowid FROM goals_fts_map WHERE goal_id = old.id);
    DELETE FROM goals_fts_map WHERE goal_id = old.id;
END;

CREATE TRIGGER goals_fts_update AFTER UPDATE ON goals BEGIN
    DELETE FROM goals_fts WHERE rowid = (SELECT rowid FROM goals_fts_map WHERE goal_id = old.id);
    INSERT INTO goals_fts(rowid, name, description) 
    VALUES ((SELECT rowid FROM goals_fts_map WHERE goal_id = new.id), new.name, COALESCE(new.description, ''));
END;

-- Tags triggers
CREATE TRIGGER tags_fts_insert AFTER INSERT ON tags BEGIN
    INSERT INTO tags_fts_map(tag_id) VALUES (new.id);
    INSERT INTO tags_fts(rowid, name) 
    VALUES ((SELECT rowid FROM tags_fts_map WHERE tag_id = new.id), new.name);
END;

CREATE TRIGGER tags_fts_delete AFTER DELETE ON tags BEGIN
    DELETE FROM tags_fts WHERE rowid = (SELECT rowid FROM tags_fts_map WHERE tag_id = old.id);
    DELETE FROM tags_fts_map WHERE tag_id = old.id;
END;

CREATE TRIGGER tags_fts_update AFTER UPDATE ON tags BEGIN
    DELETE FROM tags_fts WHERE rowid = (SELECT rowid FROM tags_fts_map WHERE tag_id = old.id);
    INSERT INTO tags_fts(rowid, name) 
    VALUES ((SELECT rowid FROM tags_fts_map WHERE tag_id = new.id), new.name);
END;

-- Bookmarks triggers
CREATE TRIGGER bookmarks_fts_insert AFTER INSERT ON bookmarks BEGIN
    INSERT INTO bookmarks_fts_map(bookmark_id) VALUES (new.id);
    INSERT INTO bookmarks_fts(rowid, title, description, site_name, author)
    VALUES ((SELECT rowid FROM bookmarks_fts_map WHERE bookmark_id = new.id), 
            COALESCE(new.title, ''), COALESCE(new.description, ''), 
            COALESCE(new.site_name, ''), COALESCE(new.author, ''));
END;

CREATE TRIGGER bookmarks_fts_delete AFTER DELETE ON bookmarks BEGIN
    DELETE FROM bookmarks_fts WHERE rowid = (SELECT rowid FROM bookmarks_fts_map WHERE bookmark_id = old.id);
    DELETE FROM bookmarks_fts_map WHERE bookmark_id = old.id;
END;

CREATE TRIGGER bookmarks_fts_update AFTER UPDATE ON bookmarks BEGIN
    DELETE FROM bookmarks_fts WHERE rowid = (SELECT rowid FROM bookmarks_fts_map WHERE bookmark_id = old.id);
    INSERT INTO bookmarks_fts(rowid, title, description, site_name, author)
    VALUES ((SELECT rowid FROM bookmarks_fts_map WHERE bookmark_id = new.id), 
            COALESCE(new.title, ''), COALESCE(new.description, ''), 
            COALESCE(new.site_name, ''), COALESCE(new.author, ''));
END;

-- Re-backfill existing data into FTS indexes
INSERT INTO entries_fts(rowid, document) 
SELECT m.rowid, e.document 
FROM entries e 
JOIN entries_fts_map m ON e.id = m.entry_id 
WHERE e.deleted_at IS NULL;

INSERT INTO tasks_fts(rowid, title, description) 
SELECT m.rowid, t.title, COALESCE(t.description, '') 
FROM tasks t 
JOIN tasks_fts_map m ON t.id = m.task_id 
WHERE t.deleted_at IS NULL;

INSERT INTO subtasks_fts(rowid, title) 
SELECT m.rowid, s.title 
FROM subtasks s 
JOIN subtasks_fts_map m ON s.id = m.subtask_id 
WHERE s.deleted_at IS NULL;

INSERT INTO goals_fts(rowid, name, description) 
SELECT m.rowid, g.name, COALESCE(g.description, '') 
FROM goals g 
JOIN goals_fts_map m ON g.id = m.goal_id 
WHERE g.deleted_at IS NULL;

INSERT INTO tags_fts(rowid, name) 
SELECT m.rowid, t.name 
FROM tags t 
JOIN tags_fts_map m ON t.id = m.tag_id 
WHERE t.deleted_at IS NULL;

INSERT INTO bookmarks_fts(rowid, title, description, site_name, author)
SELECT m.rowid, COALESCE(b.title, ''), COALESCE(b.description, ''), 
       COALESCE(b.site_name, ''), COALESCE(b.author, '')
FROM bookmarks b
JOIN bookmarks_fts_map m ON b.id = m.bookmark_id
WHERE b.deleted_at IS NULL;
