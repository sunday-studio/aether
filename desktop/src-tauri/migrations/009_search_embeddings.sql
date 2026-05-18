-- Local embedding storage for semantic search and future RAG.
-- This table is derived from search_documents and intentionally does not
-- participate in sync. Each device rebuilds embeddings from local source data.

CREATE TABLE IF NOT EXISTS search_embeddings (
    id TEXT PRIMARY KEY,
    search_document_id TEXT NOT NULL,
    model_name TEXT NOT NULL,
    dimensions INTEGER NOT NULL,
    vector BLOB NOT NULL,
    text_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(search_document_id, model_name),
    FOREIGN KEY(search_document_id) REFERENCES search_documents(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_search_embeddings_document
    ON search_embeddings(search_document_id);

CREATE INDEX IF NOT EXISTS idx_search_embeddings_model
    ON search_embeddings(model_name);

CREATE INDEX IF NOT EXISTS idx_search_embeddings_text_hash
    ON search_embeddings(text_hash);

CREATE TRIGGER IF NOT EXISTS search_embeddings_delete_document
AFTER DELETE ON search_documents
BEGIN
    DELETE FROM search_embeddings WHERE search_document_id = old.id;
END;
