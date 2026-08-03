-- Sync changes whose parents have not arrived yet. These records are local-only
-- and are retried after every pull batch.
CREATE TABLE IF NOT EXISTS _sync_deferred_changes (
    entity TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    change_json TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    last_error TEXT NOT NULL,
    PRIMARY KEY (entity, entity_id)
);

CREATE INDEX IF NOT EXISTS idx_sync_deferred_changes_updated_at
    ON _sync_deferred_changes(updated_at);
