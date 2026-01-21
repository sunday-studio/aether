-- Add vector embedding columns for semantic similarity search
-- Using F32_BLOB(384) for 384-dimensional embeddings (all-MiniLM-L6-v2 model)

-- Add embedding columns to searchable resources
ALTER TABLE entries ADD COLUMN embedding F32_BLOB(384);
ALTER TABLE tasks ADD COLUMN embedding F32_BLOB(384);
ALTER TABLE subtasks ADD COLUMN embedding F32_BLOB(384);
ALTER TABLE goals ADD COLUMN embedding F32_BLOB(384);
ALTER TABLE tags ADD COLUMN embedding F32_BLOB(384);

-- Create vector indexes using libsql_vector_idx for similarity search
-- Using cosine distance metric for semantic similarity
CREATE INDEX IF NOT EXISTS entries_embedding_idx 
    ON entries(libsql_vector_idx(embedding, 'metric=cosine'));

CREATE INDEX IF NOT EXISTS tasks_embedding_idx 
    ON tasks(libsql_vector_idx(embedding, 'metric=cosine'));

CREATE INDEX IF NOT EXISTS subtasks_embedding_idx 
    ON subtasks(libsql_vector_idx(embedding, 'metric=cosine'));

CREATE INDEX IF NOT EXISTS goals_embedding_idx 
    ON goals(libsql_vector_idx(embedding, 'metric=cosine'));

CREATE INDEX IF NOT EXISTS tags_embedding_idx 
    ON tags(libsql_vector_idx(embedding, 'metric=cosine'));
