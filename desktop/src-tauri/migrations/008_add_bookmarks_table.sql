-- Bookmarks table
CREATE TABLE IF NOT EXISTS bookmarks (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL UNIQUE,
    title TEXT,
    description TEXT,
    image_url TEXT,
    favicon_url TEXT,
    site_name TEXT,
    author TEXT,
    published_at TEXT,
    content_type TEXT, -- 'article', 'video', 'tweet', 'image', etc.
    metadata_json TEXT, -- Full metadata as JSON for extensibility
    is_archived INTEGER NOT NULL DEFAULT 0,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_bookmarks_url ON bookmarks(url);
CREATE INDEX IF NOT EXISTS idx_bookmarks_created_at ON bookmarks(created_at);
CREATE INDEX IF NOT EXISTS idx_bookmarks_deleted_at ON bookmarks(deleted_at);
CREATE INDEX IF NOT EXISTS idx_bookmarks_content_type ON bookmarks(content_type);

-- FTS5 index for search
CREATE VIRTUAL TABLE IF NOT EXISTS bookmarks_fts USING fts5(
    title,
    description,
    site_name,
    author,
    tokenize='trigram',
    detail='column'
);

-- Triggers to keep FTS index in sync
CREATE TRIGGER IF NOT EXISTS bookmarks_fts_insert AFTER INSERT ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(rowid, title, description, site_name, author)
    VALUES (new.id, COALESCE(new.title, ''), COALESCE(new.description, ''), 
            COALESCE(new.site_name, ''), COALESCE(new.author, ''));
END;

CREATE TRIGGER IF NOT EXISTS bookmarks_fts_delete AFTER DELETE ON bookmarks BEGIN
    DELETE FROM bookmarks_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS bookmarks_fts_update AFTER UPDATE ON bookmarks BEGIN
    DELETE FROM bookmarks_fts WHERE rowid = old.id;
    INSERT INTO bookmarks_fts(rowid, title, description, site_name, author)
    VALUES (new.id, COALESCE(new.title, ''), COALESCE(new.description, ''), 
            COALESCE(new.site_name, ''), COALESCE(new.author, ''));
END;

-- Bookmark-Tag many-to-many relationship
CREATE TABLE IF NOT EXISTS bookmark_tags (
    bookmark_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (bookmark_id, tag_id),
    FOREIGN KEY (bookmark_id) REFERENCES bookmarks(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Add embedding column for semantic search
ALTER TABLE bookmarks ADD COLUMN embedding F32_BLOB(384);

-- Create vector index
CREATE INDEX IF NOT EXISTS bookmarks_embedding_idx 
    ON bookmarks(libsql_vector_idx(embedding, 'metric=cosine'));

-- Backfill existing bookmarks into FTS (if any exist)
INSERT INTO bookmarks_fts(rowid, title, description, site_name, author)
SELECT id, COALESCE(title, ''), COALESCE(description, ''), 
       COALESCE(site_name, ''), COALESCE(author, '')
FROM bookmarks WHERE deleted_at IS NULL;
