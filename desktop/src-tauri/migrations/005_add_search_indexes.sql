-- Add FTS5 search indexes with trigram tokenizer for fuzzy search
-- This enables typo-tolerant substring matching across all resources

-- Entries FTS index
CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
    document,
    tokenize='trigram',
    detail='column'
);

-- Tasks FTS index (title + description)
CREATE VIRTUAL TABLE IF NOT EXISTS tasks_fts USING fts5(
    title,
    description,
    tokenize='trigram',
    detail='column'
);

-- Subtasks FTS index
CREATE VIRTUAL TABLE IF NOT EXISTS subtasks_fts USING fts5(
    title,
    tokenize='trigram',
    detail='column'
);

-- Goals FTS index (name + description)
CREATE VIRTUAL TABLE IF NOT EXISTS goals_fts USING fts5(
    name,
    description,
    tokenize='trigram',
    detail='column'
);

-- Tags FTS index
CREATE VIRTUAL TABLE IF NOT EXISTS tags_fts USING fts5(
    name,
    tokenize='trigram',
    detail='column'
);

-- Triggers to keep FTS indexes in sync with base tables

-- Entries triggers
CREATE TRIGGER IF NOT EXISTS entries_fts_insert AFTER INSERT ON entries BEGIN
    INSERT INTO entries_fts(rowid, document) VALUES (new.id, new.document);
END;

CREATE TRIGGER IF NOT EXISTS entries_fts_delete AFTER DELETE ON entries BEGIN
    DELETE FROM entries_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS entries_fts_update AFTER UPDATE ON entries BEGIN
    DELETE FROM entries_fts WHERE rowid = old.id;
    INSERT INTO entries_fts(rowid, document) VALUES (new.id, new.document);
END;

-- Tasks triggers
CREATE TRIGGER IF NOT EXISTS tasks_fts_insert AFTER INSERT ON tasks BEGIN
    INSERT INTO tasks_fts(rowid, title, description) 
    VALUES (new.id, new.title, COALESCE(new.description, ''));
END;

CREATE TRIGGER IF NOT EXISTS tasks_fts_delete AFTER DELETE ON tasks BEGIN
    DELETE FROM tasks_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS tasks_fts_update AFTER UPDATE ON tasks BEGIN
    DELETE FROM tasks_fts WHERE rowid = old.id;
    INSERT INTO tasks_fts(rowid, title, description) 
    VALUES (new.id, new.title, COALESCE(new.description, ''));
END;

-- Subtasks triggers
CREATE TRIGGER IF NOT EXISTS subtasks_fts_insert AFTER INSERT ON subtasks BEGIN
    INSERT INTO subtasks_fts(rowid, title) VALUES (new.id, new.title);
END;

CREATE TRIGGER IF NOT EXISTS subtasks_fts_delete AFTER DELETE ON subtasks BEGIN
    DELETE FROM subtasks_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS subtasks_fts_update AFTER UPDATE ON subtasks BEGIN
    DELETE FROM subtasks_fts WHERE rowid = old.id;
    INSERT INTO subtasks_fts(rowid, title) VALUES (new.id, new.title);
END;

-- Goals triggers
CREATE TRIGGER IF NOT EXISTS goals_fts_insert AFTER INSERT ON goals BEGIN
    INSERT INTO goals_fts(rowid, name, description) 
    VALUES (new.id, new.name, COALESCE(new.description, ''));
END;

CREATE TRIGGER IF NOT EXISTS goals_fts_delete AFTER DELETE ON goals BEGIN
    DELETE FROM goals_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS goals_fts_update AFTER UPDATE ON goals BEGIN
    DELETE FROM goals_fts WHERE rowid = old.id;
    INSERT INTO goals_fts(rowid, name, description) 
    VALUES (new.id, new.name, COALESCE(new.description, ''));
END;

-- Tags triggers
CREATE TRIGGER IF NOT EXISTS tags_fts_insert AFTER INSERT ON tags BEGIN
    INSERT INTO tags_fts(rowid, name) VALUES (new.id, new.name);
END;

CREATE TRIGGER IF NOT EXISTS tags_fts_delete AFTER DELETE ON tags BEGIN
    DELETE FROM tags_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS tags_fts_update AFTER UPDATE ON tags BEGIN
    DELETE FROM tags_fts WHERE rowid = old.id;
    INSERT INTO tags_fts(rowid, name) VALUES (new.id, new.name);
END;

-- Backfill existing data into FTS indexes
INSERT INTO entries_fts(rowid, document) 
SELECT id, document FROM entries WHERE deleted_at IS NULL;

INSERT INTO tasks_fts(rowid, title, description) 
SELECT id, title, COALESCE(description, '') FROM tasks WHERE deleted_at IS NULL;

INSERT INTO subtasks_fts(rowid, title) 
SELECT id, title FROM subtasks WHERE deleted_at IS NULL;

INSERT INTO goals_fts(rowid, name, description) 
SELECT id, name, COALESCE(description, '') FROM goals WHERE deleted_at IS NULL;

INSERT INTO tags_fts(rowid, name) 
SELECT id, name FROM tags WHERE deleted_at IS NULL;
