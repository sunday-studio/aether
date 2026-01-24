-- Add sync columns to all synced tables for offline-first E2E sync.
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
