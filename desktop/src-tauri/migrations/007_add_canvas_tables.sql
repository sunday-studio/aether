-- Migration: Add canvas tables
-- Version: 007
-- Description: Creates canvases table for storing JSON Canvas-compliant canvas data

CREATE TABLE IF NOT EXISTS canvases (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    canvas_data TEXT NOT NULL, -- JSON Canvas format
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_canvases_created_at ON canvases(created_at);
CREATE INDEX IF NOT EXISTS idx_canvases_updated_at ON canvases(updated_at);
CREATE INDEX IF NOT EXISTS idx_canvases_deleted_at ON canvases(deleted_at);
