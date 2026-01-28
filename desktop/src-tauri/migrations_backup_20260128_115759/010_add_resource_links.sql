-- Migration: Add resource links table
-- Version: 010
-- Description: Creates resource_links table for bidirectional linking between all resources

CREATE TABLE IF NOT EXISTS resource_links (
    id TEXT PRIMARY KEY,
    source_type TEXT NOT NULL, -- 'entry', 'task', 'goal', 'canvas', 'bookmark'
    source_id TEXT NOT NULL,
    target_type TEXT NOT NULL, -- 'entry', 'task', 'goal', 'canvas', 'bookmark'
    target_id TEXT NOT NULL,
    link_text TEXT, -- Optional display text for the link
    created_at TEXT NOT NULL,
    UNIQUE(source_type, source_id, target_type, target_id)
);

-- Indexes for efficient backlink queries
CREATE INDEX IF NOT EXISTS idx_resource_links_source ON resource_links(source_type, source_id);
CREATE INDEX IF NOT EXISTS idx_resource_links_target ON resource_links(target_type, target_id);
CREATE INDEX IF NOT EXISTS idx_resource_links_created_at ON resource_links(created_at);
