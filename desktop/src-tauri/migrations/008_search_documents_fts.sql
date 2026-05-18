-- FTS-backed keyword search over the canonical local search index.
-- This is local derived data; sync continues to move only encrypted source data.

CREATE VIRTUAL TABLE IF NOT EXISTS search_documents_fts USING fts5(
    title,
    text,
    resource_type UNINDEXED,
    content='search_documents',
    content_rowid='rowid',
    tokenize='trigram',
    detail='full'
);

INSERT INTO search_documents_fts(rowid, title, text, resource_type)
SELECT rowid, title, text, resource_type
FROM search_documents
WHERE rowid NOT IN (
    SELECT rowid FROM search_documents_fts
);

CREATE TRIGGER IF NOT EXISTS search_documents_fts_insert
AFTER INSERT ON search_documents
BEGIN
    INSERT INTO search_documents_fts(rowid, title, text, resource_type)
    VALUES (new.rowid, new.title, new.text, new.resource_type);
END;

CREATE TRIGGER IF NOT EXISTS search_documents_fts_update
AFTER UPDATE ON search_documents
BEGIN
    INSERT INTO search_documents_fts(search_documents_fts, rowid, title, text, resource_type)
    VALUES ('delete', old.rowid, old.title, old.text, old.resource_type);
    INSERT INTO search_documents_fts(rowid, title, text, resource_type)
    VALUES (new.rowid, new.title, new.text, new.resource_type);
END;

CREATE TRIGGER IF NOT EXISTS search_documents_fts_delete
AFTER DELETE ON search_documents
BEGIN
    INSERT INTO search_documents_fts(search_documents_fts, rowid, title, text, resource_type)
    VALUES ('delete', old.rowid, old.title, old.text, old.resource_type);
END;
