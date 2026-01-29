-- Media handling with entity-agnostic structure from the start
-- Supports entries, canvases, bookmarks, tasks from day one

-- Media items table (entity-agnostic from start)
CREATE TABLE IF NOT EXISTS media_items (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL CHECK(entity_type IN ('entry', 'canvas', 'bookmark', 'task')),
    entity_id TEXT NOT NULL,
    media_type TEXT NOT NULL CHECK(media_type IN ('audio', 'image', 'video')),
    file_path TEXT NOT NULL,
    metadata TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_media_entity ON media_items(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_media_type ON media_items(media_type);

-- Audio transcriptions table
CREATE TABLE IF NOT EXISTS audio_transcriptions (
    id TEXT PRIMARY KEY,
    media_id TEXT NOT NULL,
    transcription_text TEXT NOT NULL,
    provider TEXT NOT NULL,
    provider_config TEXT,
    confidence_score REAL,
    status TEXT NOT NULL CHECK(status IN ('pending', 'processing', 'complete', 'failed')),
    error_message TEXT,
    is_active INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (media_id) REFERENCES media_items(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_transcriptions_media_id ON audio_transcriptions(media_id);
CREATE INDEX IF NOT EXISTS idx_transcriptions_status ON audio_transcriptions(status);
