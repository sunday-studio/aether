-- Complete sync system in one migration
-- Adds sync infrastructure, columns, indexes, and triggers for offline-first E2E sync

-- ============================================================================
-- SYNC INFRASTRUCTURE TABLES
-- ============================================================================

-- Sync outbox for queuing changes to push
CREATE TABLE IF NOT EXISTS _sync_outbox (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    op TEXT NOT NULL CHECK(op IN ('upsert', 'delete')),
    queued_at INTEGER NOT NULL,
    UNIQUE(entity, entity_id)
);

CREATE INDEX IF NOT EXISTS idx_sync_outbox_queued ON _sync_outbox(queued_at);

-- Sync metadata storage (device_id, server_url, last_sync, key_salt, key_check, etc.)
CREATE TABLE IF NOT EXISTS _sync_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Unknown entities from newer clients (forward compatibility)
CREATE TABLE IF NOT EXISTS _sync_unknown (
    entity TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    data TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (entity, entity_id)
);

-- Initialize suppress_triggers flag (must exist for triggers to work)
INSERT OR IGNORE INTO _sync_meta (key, value) VALUES ('_suppress_triggers', '0');

-- ============================================================================
-- SYNC COLUMNS ON ALL TABLES
-- ============================================================================
-- _sync_id: stable identity (matches id for entities; composite for junctions)
-- _updated_at: ms since epoch for last-write-wins
-- _deleted: soft delete flag (0/1)
-- _extra: forward-compat JSON
-- _version: optional optimistic locking

-- entries
ALTER TABLE entries ADD COLUMN _sync_id TEXT;
ALTER TABLE entries ADD COLUMN _updated_at INTEGER;
ALTER TABLE entries ADD COLUMN _deleted INTEGER DEFAULT 0;
ALTER TABLE entries ADD COLUMN _extra TEXT DEFAULT '{}';
ALTER TABLE entries ADD COLUMN _version INTEGER DEFAULT 1;
UPDATE entries SET _sync_id = id, _updated_at = COALESCE(CAST(strftime('%s', updated_at) AS INTEGER) * 1000, CAST(strftime('%s', created_at) AS INTEGER) * 1000), _deleted = CASE WHEN is_deleted = 1 OR deleted_at IS NOT NULL THEN 1 ELSE 0 END WHERE _sync_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_entries_sync_id ON entries(_sync_id);

-- tags
ALTER TABLE tags ADD COLUMN _sync_id TEXT;
ALTER TABLE tags ADD COLUMN _updated_at INTEGER;
ALTER TABLE tags ADD COLUMN _deleted INTEGER DEFAULT 0;
ALTER TABLE tags ADD COLUMN _extra TEXT DEFAULT '{}';
ALTER TABLE tags ADD COLUMN _version INTEGER DEFAULT 1;
UPDATE tags SET _sync_id = id, _updated_at = COALESCE(CAST(strftime('%s', updated_at) AS INTEGER) * 1000, CAST(strftime('%s', created_at) AS INTEGER) * 1000), _deleted = CASE WHEN deleted_at IS NOT NULL THEN 1 ELSE 0 END WHERE _sync_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_tags_sync_id ON tags(_sync_id);

-- entry_tags
ALTER TABLE entry_tags ADD COLUMN _sync_id TEXT;
ALTER TABLE entry_tags ADD COLUMN _updated_at INTEGER;
ALTER TABLE entry_tags ADD COLUMN _deleted INTEGER DEFAULT 0;
ALTER TABLE entry_tags ADD COLUMN _extra TEXT DEFAULT '{}';
ALTER TABLE entry_tags ADD COLUMN _version INTEGER DEFAULT 1;
UPDATE entry_tags SET _sync_id = entry_id || '|' || tag_id, _updated_at = CAST(strftime('%s', 'now') AS INTEGER) * 1000, _deleted = 0 WHERE _sync_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_entry_tags_sync_id ON entry_tags(_sync_id);

-- goals
ALTER TABLE goals ADD COLUMN _sync_id TEXT;
ALTER TABLE goals ADD COLUMN _updated_at INTEGER;
ALTER TABLE goals ADD COLUMN _deleted INTEGER DEFAULT 0;
ALTER TABLE goals ADD COLUMN _extra TEXT DEFAULT '{}';
ALTER TABLE goals ADD COLUMN _version INTEGER DEFAULT 1;
UPDATE goals SET _sync_id = id, _updated_at = COALESCE(CAST(strftime('%s', updated_at) AS INTEGER) * 1000, CAST(strftime('%s', created_at) AS INTEGER) * 1000), _deleted = CASE WHEN deleted_at IS NOT NULL THEN 1 ELSE 0 END WHERE _sync_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_goals_sync_id ON goals(_sync_id);

-- goal_tags
ALTER TABLE goal_tags ADD COLUMN _sync_id TEXT;
ALTER TABLE goal_tags ADD COLUMN _updated_at INTEGER;
ALTER TABLE goal_tags ADD COLUMN _deleted INTEGER DEFAULT 0;
ALTER TABLE goal_tags ADD COLUMN _extra TEXT DEFAULT '{}';
ALTER TABLE goal_tags ADD COLUMN _version INTEGER DEFAULT 1;
UPDATE goal_tags SET _sync_id = goal_id || '|' || tag_id, _updated_at = CAST(strftime('%s', 'now') AS INTEGER) * 1000, _deleted = 0 WHERE _sync_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_goal_tags_sync_id ON goal_tags(_sync_id);

-- goal_instances
ALTER TABLE goal_instances ADD COLUMN _sync_id TEXT;
ALTER TABLE goal_instances ADD COLUMN _updated_at INTEGER;
ALTER TABLE goal_instances ADD COLUMN _deleted INTEGER DEFAULT 0;
ALTER TABLE goal_instances ADD COLUMN _extra TEXT DEFAULT '{}';
ALTER TABLE goal_instances ADD COLUMN _version INTEGER DEFAULT 1;
UPDATE goal_instances SET _sync_id = id, _updated_at = COALESCE(CAST(strftime('%s', updated_at) AS INTEGER) * 1000, CAST(strftime('%s', created_at) AS INTEGER) * 1000), _deleted = CASE WHEN deleted_at IS NOT NULL THEN 1 ELSE 0 END WHERE _sync_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_goal_instances_sync_id ON goal_instances(_sync_id);

-- goal_instance_tags
ALTER TABLE goal_instance_tags ADD COLUMN _sync_id TEXT;
ALTER TABLE goal_instance_tags ADD COLUMN _updated_at INTEGER;
ALTER TABLE goal_instance_tags ADD COLUMN _deleted INTEGER DEFAULT 0;
ALTER TABLE goal_instance_tags ADD COLUMN _extra TEXT DEFAULT '{}';
ALTER TABLE goal_instance_tags ADD COLUMN _version INTEGER DEFAULT 1;
UPDATE goal_instance_tags SET _sync_id = goal_instance_id || '|' || tag_id, _updated_at = CAST(strftime('%s', 'now') AS INTEGER) * 1000, _deleted = 0 WHERE _sync_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_goal_instance_tags_sync_id ON goal_instance_tags(_sync_id);

-- tasks
ALTER TABLE tasks ADD COLUMN _sync_id TEXT;
ALTER TABLE tasks ADD COLUMN _updated_at INTEGER;
ALTER TABLE tasks ADD COLUMN _deleted INTEGER DEFAULT 0;
ALTER TABLE tasks ADD COLUMN _extra TEXT DEFAULT '{}';
ALTER TABLE tasks ADD COLUMN _version INTEGER DEFAULT 1;
UPDATE tasks SET _sync_id = id, _updated_at = COALESCE(CAST(strftime('%s', updated_at) AS INTEGER) * 1000, CAST(strftime('%s', created_at) AS INTEGER) * 1000), _deleted = CASE WHEN deleted_at IS NOT NULL THEN 1 ELSE 0 END WHERE _sync_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_tasks_sync_id ON tasks(_sync_id);

-- task_tags
ALTER TABLE task_tags ADD COLUMN _sync_id TEXT;
ALTER TABLE task_tags ADD COLUMN _updated_at INTEGER;
ALTER TABLE task_tags ADD COLUMN _deleted INTEGER DEFAULT 0;
ALTER TABLE task_tags ADD COLUMN _extra TEXT DEFAULT '{}';
ALTER TABLE task_tags ADD COLUMN _version INTEGER DEFAULT 1;
UPDATE task_tags SET _sync_id = task_id || '|' || tag_id, _updated_at = CAST(strftime('%s', 'now') AS INTEGER) * 1000, _deleted = 0 WHERE _sync_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_task_tags_sync_id ON task_tags(_sync_id);

-- subtasks
ALTER TABLE subtasks ADD COLUMN _sync_id TEXT;
ALTER TABLE subtasks ADD COLUMN _updated_at INTEGER;
ALTER TABLE subtasks ADD COLUMN _deleted INTEGER DEFAULT 0;
ALTER TABLE subtasks ADD COLUMN _extra TEXT DEFAULT '{}';
ALTER TABLE subtasks ADD COLUMN _version INTEGER DEFAULT 1;
UPDATE subtasks SET _sync_id = id, _updated_at = COALESCE(CAST(strftime('%s', updated_at) AS INTEGER) * 1000, CAST(strftime('%s', created_at) AS INTEGER) * 1000), _deleted = CASE WHEN deleted_at IS NOT NULL THEN 1 ELSE 0 END WHERE _sync_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_subtasks_sync_id ON subtasks(_sync_id);

-- media_items
ALTER TABLE media_items ADD COLUMN _sync_id TEXT;
ALTER TABLE media_items ADD COLUMN _updated_at INTEGER;
ALTER TABLE media_items ADD COLUMN _deleted INTEGER DEFAULT 0;
ALTER TABLE media_items ADD COLUMN _extra TEXT DEFAULT '{}';
ALTER TABLE media_items ADD COLUMN _version INTEGER DEFAULT 1;
UPDATE media_items SET _sync_id = id, _updated_at = COALESCE(CAST(strftime('%s', updated_at) AS INTEGER) * 1000, CAST(strftime('%s', created_at) AS INTEGER) * 1000), _deleted = 0 WHERE _sync_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_media_items_sync_id ON media_items(_sync_id);

-- audio_transcriptions
ALTER TABLE audio_transcriptions ADD COLUMN _sync_id TEXT;
ALTER TABLE audio_transcriptions ADD COLUMN _updated_at INTEGER;
ALTER TABLE audio_transcriptions ADD COLUMN _deleted INTEGER DEFAULT 0;
ALTER TABLE audio_transcriptions ADD COLUMN _extra TEXT DEFAULT '{}';
ALTER TABLE audio_transcriptions ADD COLUMN _version INTEGER DEFAULT 1;
UPDATE audio_transcriptions SET _sync_id = id, _updated_at = CAST(strftime('%s', created_at) AS INTEGER) * 1000, _deleted = 0 WHERE _sync_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_audio_transcriptions_sync_id ON audio_transcriptions(_sync_id);

-- canvases
ALTER TABLE canvases ADD COLUMN _sync_id TEXT;
ALTER TABLE canvases ADD COLUMN _updated_at INTEGER;
ALTER TABLE canvases ADD COLUMN _deleted INTEGER DEFAULT 0;
ALTER TABLE canvases ADD COLUMN _extra TEXT DEFAULT '{}';
ALTER TABLE canvases ADD COLUMN _version INTEGER DEFAULT 1;
UPDATE canvases SET _sync_id = id, _updated_at = COALESCE(CAST(strftime('%s', updated_at) AS INTEGER) * 1000, CAST(strftime('%s', created_at) AS INTEGER) * 1000), _deleted = CASE WHEN deleted_at IS NOT NULL THEN 1 ELSE 0 END WHERE _sync_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_canvases_sync_id ON canvases(_sync_id);

-- bookmarks
ALTER TABLE bookmarks ADD COLUMN _sync_id TEXT;
ALTER TABLE bookmarks ADD COLUMN _updated_at INTEGER;
ALTER TABLE bookmarks ADD COLUMN _deleted INTEGER DEFAULT 0;
ALTER TABLE bookmarks ADD COLUMN _extra TEXT DEFAULT '{}';
ALTER TABLE bookmarks ADD COLUMN _version INTEGER DEFAULT 1;
UPDATE bookmarks SET _sync_id = id, _updated_at = COALESCE(CAST(strftime('%s', updated_at) AS INTEGER) * 1000, CAST(strftime('%s', created_at) AS INTEGER) * 1000), _deleted = CASE WHEN is_deleted = 1 OR deleted_at IS NOT NULL THEN 1 ELSE 0 END WHERE _sync_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_bookmarks_sync_id ON bookmarks(_sync_id);

-- bookmark_tags
ALTER TABLE bookmark_tags ADD COLUMN _sync_id TEXT;
ALTER TABLE bookmark_tags ADD COLUMN _updated_at INTEGER;
ALTER TABLE bookmark_tags ADD COLUMN _deleted INTEGER DEFAULT 0;
ALTER TABLE bookmark_tags ADD COLUMN _extra TEXT DEFAULT '{}';
ALTER TABLE bookmark_tags ADD COLUMN _version INTEGER DEFAULT 1;
UPDATE bookmark_tags SET _sync_id = bookmark_id || '|' || tag_id, _updated_at = CAST(strftime('%s', 'now') AS INTEGER) * 1000, _deleted = 0 WHERE _sync_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_bookmark_tags_sync_id ON bookmark_tags(_sync_id);

-- resource_links
ALTER TABLE resource_links ADD COLUMN _sync_id TEXT;
ALTER TABLE resource_links ADD COLUMN _updated_at INTEGER;
ALTER TABLE resource_links ADD COLUMN _deleted INTEGER DEFAULT 0;
ALTER TABLE resource_links ADD COLUMN _extra TEXT DEFAULT '{}';
ALTER TABLE resource_links ADD COLUMN _version INTEGER DEFAULT 1;
UPDATE resource_links SET _sync_id = id, _updated_at = CAST(strftime('%s', created_at) AS INTEGER) * 1000, _deleted = 0 WHERE _sync_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_resource_links_sync_id ON resource_links(_sync_id);

-- ============================================================================
-- SYNC TRIGGERS
-- ============================================================================
-- INSERT/UPDATE (upsert) and UPDATE (_deleted 0->1) feed the outbox.
-- When applying remote changes, set _sync_meta key '_suppress_triggers' = '1' to avoid re-queuing.

-- entries
CREATE TRIGGER IF NOT EXISTS entries_sync_insert AFTER INSERT ON entries
WHEN (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('entries', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS entries_sync_update AFTER UPDATE ON entries
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL))
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('entries', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS entries_sync_delete AFTER UPDATE ON entries
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL)
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('entries', COALESCE(NEW._sync_id, NEW.id), 'delete', (strftime('%s','now') * 1000));
END;

-- tags
CREATE TRIGGER IF NOT EXISTS tags_sync_insert AFTER INSERT ON tags
WHEN (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('tags', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS tags_sync_update AFTER UPDATE ON tags
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL))
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('tags', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS tags_sync_delete AFTER UPDATE ON tags
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL)
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('tags', COALESCE(NEW._sync_id, NEW.id), 'delete', (strftime('%s','now') * 1000));
END;

-- entry_tags
CREATE TRIGGER IF NOT EXISTS entry_tags_sync_insert AFTER INSERT ON entry_tags
WHEN (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('entry_tags', COALESCE(NEW._sync_id, NEW.entry_id||'|'||NEW.tag_id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS entry_tags_sync_update AFTER UPDATE ON entry_tags
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL))
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('entry_tags', COALESCE(NEW._sync_id, NEW.entry_id||'|'||NEW.tag_id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS entry_tags_sync_delete AFTER UPDATE ON entry_tags
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL)
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('entry_tags', COALESCE(NEW._sync_id, NEW.entry_id||'|'||NEW.tag_id), 'delete', (strftime('%s','now') * 1000));
END;

-- goals
CREATE TRIGGER IF NOT EXISTS goals_sync_insert AFTER INSERT ON goals
WHEN (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('goals', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS goals_sync_update AFTER UPDATE ON goals
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL))
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('goals', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS goals_sync_delete AFTER UPDATE ON goals
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL)
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('goals', COALESCE(NEW._sync_id, NEW.id), 'delete', (strftime('%s','now') * 1000));
END;

-- goal_tags
CREATE TRIGGER IF NOT EXISTS goal_tags_sync_insert AFTER INSERT ON goal_tags
WHEN (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('goal_tags', COALESCE(NEW._sync_id, NEW.goal_id||'|'||NEW.tag_id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS goal_tags_sync_update AFTER UPDATE ON goal_tags
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL))
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('goal_tags', COALESCE(NEW._sync_id, NEW.goal_id||'|'||NEW.tag_id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS goal_tags_sync_delete AFTER UPDATE ON goal_tags
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL)
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('goal_tags', COALESCE(NEW._sync_id, NEW.goal_id||'|'||NEW.tag_id), 'delete', (strftime('%s','now') * 1000));
END;

-- goal_instances
CREATE TRIGGER IF NOT EXISTS goal_instances_sync_insert AFTER INSERT ON goal_instances
WHEN (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('goal_instances', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS goal_instances_sync_update AFTER UPDATE ON goal_instances
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL))
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('goal_instances', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS goal_instances_sync_delete AFTER UPDATE ON goal_instances
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL)
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('goal_instances', COALESCE(NEW._sync_id, NEW.id), 'delete', (strftime('%s','now') * 1000));
END;

-- goal_instance_tags
CREATE TRIGGER IF NOT EXISTS goal_instance_tags_sync_insert AFTER INSERT ON goal_instance_tags
WHEN (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('goal_instance_tags', COALESCE(NEW._sync_id, NEW.goal_instance_id||'|'||NEW.tag_id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS goal_instance_tags_sync_update AFTER UPDATE ON goal_instance_tags
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL))
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('goal_instance_tags', COALESCE(NEW._sync_id, NEW.goal_instance_id||'|'||NEW.tag_id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS goal_instance_tags_sync_delete AFTER UPDATE ON goal_instance_tags
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL)
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('goal_instance_tags', COALESCE(NEW._sync_id, NEW.goal_instance_id||'|'||NEW.tag_id), 'delete', (strftime('%s','now') * 1000));
END;

-- tasks
CREATE TRIGGER IF NOT EXISTS tasks_sync_insert AFTER INSERT ON tasks
WHEN (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('tasks', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS tasks_sync_update AFTER UPDATE ON tasks
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL))
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('tasks', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS tasks_sync_delete AFTER UPDATE ON tasks
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL)
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('tasks', COALESCE(NEW._sync_id, NEW.id), 'delete', (strftime('%s','now') * 1000));
END;

-- task_tags
CREATE TRIGGER IF NOT EXISTS task_tags_sync_insert AFTER INSERT ON task_tags
WHEN (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('task_tags', COALESCE(NEW._sync_id, NEW.task_id||'|'||NEW.tag_id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS task_tags_sync_update AFTER UPDATE ON task_tags
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL))
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('task_tags', COALESCE(NEW._sync_id, NEW.task_id||'|'||NEW.tag_id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS task_tags_sync_delete AFTER UPDATE ON task_tags
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL)
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('task_tags', COALESCE(NEW._sync_id, NEW.task_id||'|'||NEW.tag_id), 'delete', (strftime('%s','now') * 1000));
END;

-- subtasks
CREATE TRIGGER IF NOT EXISTS subtasks_sync_insert AFTER INSERT ON subtasks
WHEN (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('subtasks', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS subtasks_sync_update AFTER UPDATE ON subtasks
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL))
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('subtasks', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS subtasks_sync_delete AFTER UPDATE ON subtasks
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL)
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('subtasks', COALESCE(NEW._sync_id, NEW.id), 'delete', (strftime('%s','now') * 1000));
END;

-- media_items
CREATE TRIGGER IF NOT EXISTS media_items_sync_insert AFTER INSERT ON media_items
WHEN (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('media_items', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS media_items_sync_update AFTER UPDATE ON media_items
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL))
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('media_items', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS media_items_sync_delete AFTER UPDATE ON media_items
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL)
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('media_items', COALESCE(NEW._sync_id, NEW.id), 'delete', (strftime('%s','now') * 1000));
END;

-- audio_transcriptions
CREATE TRIGGER IF NOT EXISTS audio_transcriptions_sync_insert AFTER INSERT ON audio_transcriptions
WHEN (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('audio_transcriptions', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS audio_transcriptions_sync_update AFTER UPDATE ON audio_transcriptions
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL))
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('audio_transcriptions', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS audio_transcriptions_sync_delete AFTER UPDATE ON audio_transcriptions
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL)
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('audio_transcriptions', COALESCE(NEW._sync_id, NEW.id), 'delete', (strftime('%s','now') * 1000));
END;

-- canvases
CREATE TRIGGER IF NOT EXISTS canvases_sync_insert AFTER INSERT ON canvases
WHEN (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('canvases', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS canvases_sync_update AFTER UPDATE ON canvases
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL))
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('canvases', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS canvases_sync_delete AFTER UPDATE ON canvases
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL)
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('canvases', COALESCE(NEW._sync_id, NEW.id), 'delete', (strftime('%s','now') * 1000));
END;

-- bookmarks
CREATE TRIGGER IF NOT EXISTS bookmarks_sync_insert AFTER INSERT ON bookmarks
WHEN (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('bookmarks', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS bookmarks_sync_update AFTER UPDATE ON bookmarks
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL))
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('bookmarks', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS bookmarks_sync_delete AFTER UPDATE ON bookmarks
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL)
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('bookmarks', COALESCE(NEW._sync_id, NEW.id), 'delete', (strftime('%s','now') * 1000));
END;

-- bookmark_tags
CREATE TRIGGER IF NOT EXISTS bookmark_tags_sync_insert AFTER INSERT ON bookmark_tags
WHEN (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('bookmark_tags', COALESCE(NEW._sync_id, NEW.bookmark_id||'|'||NEW.tag_id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS bookmark_tags_sync_update AFTER UPDATE ON bookmark_tags
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL))
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('bookmark_tags', COALESCE(NEW._sync_id, NEW.bookmark_id||'|'||NEW.tag_id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS bookmark_tags_sync_delete AFTER UPDATE ON bookmark_tags
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL)
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('bookmark_tags', COALESCE(NEW._sync_id, NEW.bookmark_id||'|'||NEW.tag_id), 'delete', (strftime('%s','now') * 1000));
END;

-- resource_links
CREATE TRIGGER IF NOT EXISTS resource_links_sync_insert AFTER INSERT ON resource_links
WHEN (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('resource_links', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS resource_links_sync_update AFTER UPDATE ON resource_links
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL))
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('resource_links', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS resource_links_sync_delete AFTER UPDATE ON resource_links
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL)
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('resource_links', COALESCE(NEW._sync_id, NEW.id), 'delete', (strftime('%s','now') * 1000));
END;

-- ============================================================================
-- ACTIVITIES SYNC (audit log sync across devices)
-- ============================================================================

-- activities
ALTER TABLE activities ADD COLUMN _sync_id TEXT;
ALTER TABLE activities ADD COLUMN _updated_at INTEGER;
ALTER TABLE activities ADD COLUMN _deleted INTEGER DEFAULT 0;
ALTER TABLE activities ADD COLUMN _extra TEXT DEFAULT '{}';
ALTER TABLE activities ADD COLUMN _version INTEGER DEFAULT 1;
UPDATE activities SET _sync_id = id, _updated_at = CAST(strftime('%s', created_at) AS INTEGER) * 1000, _deleted = 0 WHERE _sync_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_activities_sync_id ON activities(_sync_id);

-- activities triggers
CREATE TRIGGER IF NOT EXISTS activities_sync_insert AFTER INSERT ON activities
WHEN (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('activities', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS activities_sync_update AFTER UPDATE ON activities
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL))
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('activities', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;

CREATE TRIGGER IF NOT EXISTS activities_sync_delete AFTER UPDATE ON activities
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL)
    AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('activities', COALESCE(NEW._sync_id, NEW.id), 'delete', (strftime('%s','now') * 1000));
END;
