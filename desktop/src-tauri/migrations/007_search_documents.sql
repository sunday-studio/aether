-- Canonical local search document index.
-- This table is derived from user resources and intentionally does not
-- participate in sync. Each device rebuilds it from local source data.

CREATE TABLE IF NOT EXISTS search_documents (
    id TEXT PRIMARY KEY,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    chunk_index INTEGER NOT NULL DEFAULT 0,
    title TEXT NOT NULL DEFAULT '',
    text TEXT NOT NULL DEFAULT '',
    text_hash TEXT NOT NULL,
    source_updated_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(resource_type, resource_id, chunk_index)
);

CREATE INDEX IF NOT EXISTS idx_search_documents_resource
    ON search_documents(resource_type, resource_id);

CREATE INDEX IF NOT EXISTS idx_search_documents_type
    ON search_documents(resource_type);

CREATE INDEX IF NOT EXISTS idx_search_documents_source_updated
    ON search_documents(source_updated_at);

