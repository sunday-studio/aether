-- Vector embedding structure (commented out for future use)
-- 
-- To enable vector embeddings:
-- 1. Uncomment the ALTER TABLE statements below to add embedding columns
-- 2. Uncomment the CREATE INDEX statements to create vector indexes
-- 3. Ensure libsql_vector_idx() is available (e.g., on Turso Cloud)
-- 4. Run this migration
--
-- Note: Vector indexes require libsql_vector_idx() to be available.
-- If not available (e.g., in local libSQL), the migration will continue without them.
-- The embedding columns remain intact, so vector search using vector_distance_cos()
-- will still work without the indexes (just slower for large datasets).

-- Add vector embedding columns for semantic similarity search
-- Using F32_BLOB(384) for 384-dimensional embeddings (all-MiniLM-L6-v2 model)

-- ALTER TABLE entries ADD COLUMN embedding F32_BLOB(384);
-- ALTER TABLE tasks ADD COLUMN embedding F32_BLOB(384);
-- ALTER TABLE subtasks ADD COLUMN embedding F32_BLOB(384);
-- ALTER TABLE goals ADD COLUMN embedding F32_BLOB(384);
-- ALTER TABLE tags ADD COLUMN embedding F32_BLOB(384);
-- ALTER TABLE bookmarks ADD COLUMN embedding F32_BLOB(384);

-- Create vector indexes using libsql_vector_idx for similarity search
-- Using cosine distance metric for semantic similarity

-- CREATE INDEX IF NOT EXISTS entries_embedding_idx 
--     ON entries(libsql_vector_idx(embedding, 'metric=cosine'));

-- CREATE INDEX IF NOT EXISTS tasks_embedding_idx 
--     ON tasks(libsql_vector_idx(embedding, 'metric=cosine'));

-- CREATE INDEX IF NOT EXISTS subtasks_embedding_idx 
--     ON subtasks(libsql_vector_idx(embedding, 'metric=cosine'));

-- CREATE INDEX IF NOT EXISTS goals_embedding_idx 
--     ON goals(libsql_vector_idx(embedding, 'metric=cosine'));

-- CREATE INDEX IF NOT EXISTS tags_embedding_idx 
--     ON tags(libsql_vector_idx(embedding, 'metric=cosine'));

-- CREATE INDEX IF NOT EXISTS bookmarks_embedding_idx 
--     ON bookmarks(libsql_vector_idx(embedding, 'metric=cosine'));
