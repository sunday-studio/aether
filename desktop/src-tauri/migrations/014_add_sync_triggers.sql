-- Change-tracking triggers: queue local changes into _sync_outbox.
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
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL)) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('entries', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;
CREATE TRIGGER IF NOT EXISTS entries_sync_delete AFTER UPDATE ON entries
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
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
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL)) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('tags', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;
CREATE TRIGGER IF NOT EXISTS tags_sync_delete AFTER UPDATE ON tags
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
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
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL)) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('entry_tags', COALESCE(NEW._sync_id, NEW.entry_id||'|'||NEW.tag_id), 'upsert', (strftime('%s','now') * 1000));
END;
CREATE TRIGGER IF NOT EXISTS entry_tags_sync_delete AFTER UPDATE ON entry_tags
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
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
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL)) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('goals', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;
CREATE TRIGGER IF NOT EXISTS goals_sync_delete AFTER UPDATE ON goals
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
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
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL)) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('goal_tags', COALESCE(NEW._sync_id, NEW.goal_id||'|'||NEW.tag_id), 'upsert', (strftime('%s','now') * 1000));
END;
CREATE TRIGGER IF NOT EXISTS goal_tags_sync_delete AFTER UPDATE ON goal_tags
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
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
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL)) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('goal_instances', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;
CREATE TRIGGER IF NOT EXISTS goal_instances_sync_delete AFTER UPDATE ON goal_instances
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
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
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL)) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('goal_instance_tags', COALESCE(NEW._sync_id, NEW.goal_instance_id||'|'||NEW.tag_id), 'upsert', (strftime('%s','now') * 1000));
END;
CREATE TRIGGER IF NOT EXISTS goal_instance_tags_sync_delete AFTER UPDATE ON goal_instance_tags
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
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
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL)) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('tasks', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;
CREATE TRIGGER IF NOT EXISTS tasks_sync_delete AFTER UPDATE ON tasks
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
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
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL)) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('task_tags', COALESCE(NEW._sync_id, NEW.task_id||'|'||NEW.tag_id), 'upsert', (strftime('%s','now') * 1000));
END;
CREATE TRIGGER IF NOT EXISTS task_tags_sync_delete AFTER UPDATE ON task_tags
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
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
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL)) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('subtasks', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;
CREATE TRIGGER IF NOT EXISTS subtasks_sync_delete AFTER UPDATE ON subtasks
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
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
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL)) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('media_items', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;
CREATE TRIGGER IF NOT EXISTS media_items_sync_delete AFTER UPDATE ON media_items
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
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
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL)) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('audio_transcriptions', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;
CREATE TRIGGER IF NOT EXISTS audio_transcriptions_sync_delete AFTER UPDATE ON audio_transcriptions
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
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
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL)) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('canvases', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;
CREATE TRIGGER IF NOT EXISTS canvases_sync_delete AFTER UPDATE ON canvases
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
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
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL)) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('bookmarks', COALESCE(NEW._sync_id, NEW.id), 'upsert', (strftime('%s','now') * 1000));
END;
CREATE TRIGGER IF NOT EXISTS bookmarks_sync_delete AFTER UPDATE ON bookmarks
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
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
WHEN NEW._updated_at IS NOT NULL AND (NEW._updated_at != OLD._updated_at OR (OLD._updated_at IS NULL)) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('bookmark_tags', COALESCE(NEW._sync_id, NEW.bookmark_id||'|'||NEW.tag_id), 'upsert', (strftime('%s','now') * 1000));
END;
CREATE TRIGGER IF NOT EXISTS bookmark_tags_sync_delete AFTER UPDATE ON bookmark_tags
WHEN NEW._deleted = 1 AND (OLD._deleted = 0 OR OLD._deleted IS NULL) AND (SELECT COALESCE(value,'0') FROM _sync_meta WHERE key='_suppress_triggers') = '0'
BEGIN
    INSERT OR REPLACE INTO _sync_outbox (entity, entity_id, op, queued_at)
    VALUES ('bookmark_tags', COALESCE(NEW._sync_id, NEW.bookmark_id||'|'||NEW.tag_id), 'delete', (strftime('%s','now') * 1000));
END;
