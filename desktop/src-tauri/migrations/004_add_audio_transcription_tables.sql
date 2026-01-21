-- Migration: Add audio transcription tables and migrate settings to key-value store
-- Version: 004
-- Description: Creates media_items and audio_transcriptions tables, migrates settings table to key-value structure

-- Step 1: Backup existing timezone value from old settings table
-- (This will be done in Rust code, not SQL, as we need to read and then insert)

-- Step 2: Drop old settings table
DROP TABLE IF EXISTS settings;

-- Step 3: Create new settings table as key-value store
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_settings_key ON settings(key);

-- Step 4: Create media_items table
CREATE TABLE IF NOT EXISTS media_items (
    id TEXT PRIMARY KEY,
    entry_id TEXT NOT NULL,
    media_type TEXT NOT NULL CHECK(media_type IN ('audio', 'image', 'video')),
    file_path TEXT NOT NULL,
    metadata TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_media_entry_id ON media_items(entry_id);
CREATE INDEX IF NOT EXISTS idx_media_type ON media_items(media_type);

-- Step 5: Create audio_transcriptions table
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
