-- Migration: Make media_items entity-agnostic
-- Version: 009
-- Description: Adds entity_type column, renames entry_id to entity_id, removes foreign key constraint

-- Step 1: Add entity_type column (temporary, will be populated and made NOT NULL later)
ALTER TABLE media_items ADD COLUMN entity_type TEXT;

-- Step 2: Set entity_type to 'entry' for all existing rows
UPDATE media_items SET entity_type = 'entry' WHERE entity_type IS NULL;

-- Step 3: Create new table with correct structure
CREATE TABLE IF NOT EXISTS media_items_new (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL CHECK(entity_type IN ('entry', 'canvas', 'bookmark', 'task')),
    entity_id TEXT NOT NULL,
    media_type TEXT NOT NULL CHECK(media_type IN ('audio', 'image', 'video')),
    file_path TEXT NOT NULL,
    metadata TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Step 4: Copy data from old table to new table
INSERT INTO media_items_new (id, entity_type, entity_id, media_type, file_path, metadata, created_at, updated_at)
SELECT id, entity_type, entry_id, media_type, file_path, metadata, created_at, updated_at
FROM media_items;

-- Step 5: Drop old table
DROP TABLE media_items;

-- Step 6: Rename new table to original name
ALTER TABLE media_items_new RENAME TO media_items;

-- Step 7: Recreate indexes
CREATE INDEX IF NOT EXISTS idx_media_entity ON media_items(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_media_type ON media_items(media_type);
