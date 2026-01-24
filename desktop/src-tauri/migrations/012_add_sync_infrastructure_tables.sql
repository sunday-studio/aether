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
