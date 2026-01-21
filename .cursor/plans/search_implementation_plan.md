# Search Implementation Plan

## Overview
Build a comprehensive search system across all resources (entries, tasks, subtasks, goals, tags) with:
- **Fuzzy search**: Typo-tolerant, substring matching
- **Similar search**: Semantic similarity using embeddings
- **Extensible**: Easy to add new resource types

## Current Resources

### Searchable Resources
1. **Entries** (`entries` table)
   - `document` (TEXT) - Rich text/JSON content
   - `created_at`, `updated_at` - Temporal metadata
   - Tags via `entry_tags` junction table

2. **Tasks** (`tasks` table)
   - `title` (TEXT) - Primary searchable field
   - `description` (TEXT) - Secondary searchable field
   - Tags via `task_tags` junction table

3. **Subtasks** (`subtasks` table)
   - `title` (TEXT) - Primary searchable field

4. **Goals** (`goals` table)
   - `name` (TEXT) - Primary searchable field
   - `description` (TEXT) - Secondary searchable field
   - Tags via `goal_tags` junction table

5. **Tags** (`tags` table)
   - `name` (TEXT) - Searchable field

### Future Resources
- Bookmarks (mentioned in features)
- Any new resource types added later

## Architecture Options

### Option 1: FTS5 with Trigram Tokenizer (Recommended for Fuzzy Search)
**Pros:**
- Native SQLite/LibSQL support (no external dependencies)
- Excellent for fuzzy/substring matching
- Fast indexed LIKE queries
- Works offline (local-first compatible)
- Low latency

**Cons:**
- Doesn't handle semantic similarity (synonyms, paraphrases)
- Index size larger than standard FTS
- Substrings < 3 characters need fallback

**Best for:** Typo tolerance, substring matching, prefix search

### Option 2: Vector Embeddings (Recommended for Similar Search)
**Pros:**
- Semantic understanding (synonyms, related concepts)
- Native LibSQL/Turso support
- Can find conceptually similar content
- Works with hybrid search

**Cons:**
- Requires embedding generation (external API or local model)
- More storage overhead
- Slightly slower queries
- Requires embedding model/API

**Best for:** Finding similar content, semantic search, "find things like this"

### Option 3: Hybrid Approach (Recommended Overall)
**Pros:**
- Combines benefits of both
- Fuzzy search for exact/typo-tolerant matching
- Semantic search for conceptual similarity
- Can rank/merge results from both

**Cons:**
- More complex implementation
- Higher storage requirements
- Need to manage both indexes

**Best for:** Comprehensive search experience

## Recommended Implementation: Hybrid Approach

### Phase 1: FTS5 with Trigram (Fuzzy Search)

#### 1.1 Database Schema Changes

**Migration: `004_add_search_indexes.sql`**

```sql
-- Unified search index for all resources
-- Using FTS5 with trigram tokenizer for fuzzy matching

-- Entries FTS index
CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
    document,
    tokenize='trigram',
    detail='column'
);

-- Tasks FTS index (title + description)
CREATE VIRTUAL TABLE IF NOT EXISTS tasks_fts USING fts5(
    title,
    description,
    tokenize='trigram',
    detail='column'
);

-- Subtasks FTS index
CREATE VIRTUAL TABLE IF NOT EXISTS subtasks_fts USING fts5(
    title,
    tokenize='trigram',
    detail='column'
);

-- Goals FTS index (name + description)
CREATE VIRTUAL TABLE IF NOT EXISTS goals_fts USING fts5(
    name,
    description,
    tokenize='trigram',
    detail='column'
);

-- Tags FTS index
CREATE VIRTUAL TABLE IF NOT EXISTS tags_fts USING fts5(
    name,
    tokenize='trigram',
    detail='column'
);

-- Triggers to keep FTS indexes in sync

-- Entries triggers
CREATE TRIGGER IF NOT EXISTS entries_fts_insert AFTER INSERT ON entries BEGIN
    INSERT INTO entries_fts(rowid, document) VALUES (new.id, new.document);
END;

CREATE TRIGGER IF NOT EXISTS entries_fts_delete AFTER DELETE ON entries BEGIN
    DELETE FROM entries_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS entries_fts_update AFTER UPDATE ON entries BEGIN
    DELETE FROM entries_fts WHERE rowid = old.id;
    INSERT INTO entries_fts(rowid, document) VALUES (new.id, new.document);
END;

-- Tasks triggers
CREATE TRIGGER IF NOT EXISTS tasks_fts_insert AFTER INSERT ON tasks BEGIN
    INSERT INTO tasks_fts(rowid, title, description) 
    VALUES (new.id, new.title, COALESCE(new.description, ''));
END;

CREATE TRIGGER IF NOT EXISTS tasks_fts_delete AFTER DELETE ON tasks BEGIN
    DELETE FROM tasks_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS tasks_fts_update AFTER UPDATE ON tasks BEGIN
    DELETE FROM tasks_fts WHERE rowid = old.id;
    INSERT INTO tasks_fts(rowid, title, description) 
    VALUES (new.id, new.title, COALESCE(new.description, ''));
END;

-- Subtasks triggers
CREATE TRIGGER IF NOT EXISTS subtasks_fts_insert AFTER INSERT ON subtasks BEGIN
    INSERT INTO subtasks_fts(rowid, title) VALUES (new.id, new.title);
END;

CREATE TRIGGER IF NOT EXISTS subtasks_fts_delete AFTER DELETE ON subtasks BEGIN
    DELETE FROM subtasks_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS subtasks_fts_update AFTER UPDATE ON subtasks BEGIN
    DELETE FROM subtasks_fts WHERE rowid = old.id;
    INSERT INTO subtasks_fts(rowid, title) VALUES (new.id, new.title);
END;

-- Goals triggers
CREATE TRIGGER IF NOT EXISTS goals_fts_insert AFTER INSERT ON goals BEGIN
    INSERT INTO goals_fts(rowid, name, description) 
    VALUES (new.id, new.name, COALESCE(new.description, ''));
END;

CREATE TRIGGER IF NOT EXISTS goals_fts_delete AFTER DELETE ON goals BEGIN
    DELETE FROM goals_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS goals_fts_update AFTER UPDATE ON goals BEGIN
    DELETE FROM goals_fts WHERE rowid = old.id;
    INSERT INTO goals_fts(rowid, name, description) 
    VALUES (new.id, new.name, COALESCE(new.description, ''));
END;

-- Tags triggers
CREATE TRIGGER IF NOT EXISTS tags_fts_insert AFTER INSERT ON tags BEGIN
    INSERT INTO tags_fts(rowid, name) VALUES (new.id, new.name);
END;

CREATE TRIGGER IF NOT EXISTS tags_fts_delete AFTER DELETE ON tags BEGIN
    DELETE FROM tags_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS tags_fts_update AFTER UPDATE ON tags BEGIN
    DELETE FROM tags_fts WHERE rowid = old.id;
    INSERT INTO tags_fts(rowid, name) VALUES (new.id, new.name);
END;

-- Backfill existing data
INSERT INTO entries_fts(rowid, document) 
SELECT id, document FROM entries WHERE deleted_at IS NULL;

INSERT INTO tasks_fts(rowid, title, description) 
SELECT id, title, COALESCE(description, '') FROM tasks WHERE deleted_at IS NULL;

INSERT INTO subtasks_fts(rowid, title) 
SELECT id, title FROM subtasks WHERE deleted_at IS NULL;

INSERT INTO goals_fts(rowid, name, description) 
SELECT id, name, COALESCE(description, '') FROM goals WHERE deleted_at IS NULL;

INSERT INTO tags_fts(rowid, name) 
SELECT id, name FROM tags WHERE deleted_at IS NULL;
```

#### 1.2 Backend Implementation

**New Files:**
- `desktop/src-tauri/src/db/repositories/search.rs` - Search repository
- `desktop/src-tauri/src/handlers/search.rs` - Search API handlers
- `desktop/src-tauri/src/commands/search.rs` - Tauri commands

**Search Repository (`db/repositories/search.rs`):**
```rust
// Unified search across all resource types
// Supports:
// - Fuzzy matching via FTS5 trigram
// - Resource type filtering
// - Tag filtering
// - Date range filtering
// - Pagination
```

**Search Handler (`handlers/search.rs`):**
```rust
// GET /v1/search
// Query params:
//   - q: search query (required)
//   - types: comma-separated resource types (entry, task, subtask, goal, tag)
//   - tags: comma-separated tag IDs
//   - limit: max results (default 50)
//   - offset: pagination offset
//   - fuzzy: enable fuzzy matching (default true)
```

**Search Command (`commands/search.rs`):**
```rust
// Tauri command wrapper for search handler
```

#### 1.3 Frontend Implementation

**New Files:**
- `desktop/src/features/search/search.view.tsx` - Search UI component
- `desktop/src/features/search/search.domain.ts` - Search types/domain
- `desktop/src/hooks/use-search.ts` - Search hook

**Search UI Features:**
- Global search bar (keyboard shortcut: Cmd+K / Ctrl+K)
- Search results grouped by resource type
- Highlight matching text
- Quick filters (resource types, tags)
- Recent searches

### Phase 2: Vector Embeddings (Similar Search)

#### 2.1 Database Schema Changes

**Migration: `005_add_vector_embeddings.sql`**

```sql
-- Add embedding columns to searchable resources
ALTER TABLE entries ADD COLUMN embedding F32_BLOB(384);
ALTER TABLE tasks ADD COLUMN embedding F32_BLOB(384);
ALTER TABLE subtasks ADD COLUMN embedding F32_BLOB(384);
ALTER TABLE goals ADD COLUMN embedding F32_BLOB(384);
ALTER TABLE tags ADD COLUMN embedding F32_BLOB(384);

-- Create vector indexes for similarity search
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
```

#### 2.2 Embedding Generation

**Options:**
1. **Local Model** (recommended for privacy/offline)
   - Use `sentence-transformers` equivalent in Rust
   - Or call Python script via subprocess
   - Models: `all-MiniLM-L6-v2` (384 dims, fast, good quality)

2. **External API** (simpler, requires internet)
   - OpenAI `text-embedding-3-small` (1536 dims)
   - Cohere Embed (384 dims)
   - HuggingFace Inference API

**Implementation:**
- Background job to generate embeddings for existing content
- Generate embeddings on create/update
- Batch processing for efficiency

**New Files:**
- `desktop/src-tauri/src/utils/embeddings.rs` - Embedding generation
- `desktop/src-tauri/src/handlers/embeddings.rs` - Embedding management API

#### 2.3 Similar Search API

**New Endpoint:**
```
GET /v1/search/similar
Query params:
  - resource_type: entry | task | subtask | goal | tag
  - resource_id: ID of the resource to find similar ones
  - limit: max results (default 10)
```

**Implementation:**
```rust
// 1. Get embedding of source resource
// 2. Use vector_top_k() to find similar resources
// 3. Return ranked results
```

### Phase 3: Hybrid Search

#### 3.1 Combined Search API

**Enhanced Endpoint:**
```
GET /v1/search
Query params:
  - q: search query
  - mode: fuzzy | similar | hybrid (default: hybrid)
  - types: resource types filter
  - tags: tag filter
  - limit: max results
```

**Hybrid Ranking:**
```rust
// 1. Get fuzzy matches from FTS5 (with relevance scores)
// 2. Get semantic matches from vector search (with distance scores)
// 3. Merge and re-rank using weighted combination:
//    final_score = (fuzzy_score * fuzzy_weight) + (similarity_score * similarity_weight)
// 4. Return unified results
```

## Implementation Phases

### Phase 1: Basic Fuzzy Search (Week 1-2)
- [ ] Create FTS5 indexes with trigram tokenizer
- [ ] Implement search repository
- [ ] Create search API endpoint
- [ ] Build basic search UI
- [ ] Add keyboard shortcut (Cmd+K)
- [ ] Test with existing data

### Phase 2: Enhanced Fuzzy Search (Week 2-3)
- [ ] Add resource type filtering
- [ ] Add tag filtering
- [ ] Add date range filtering
- [ ] Improve result ranking
- [ ] Add search result highlighting
- [ ] Add recent searches

### Phase 3: Vector Embeddings Setup (Week 3-4)
- [ ] Choose embedding model/API
- [ ] Implement embedding generation
- [ ] Add embedding columns to schema
- [ ] Create vector indexes
- [ ] Background job to generate embeddings for existing data
- [ ] Generate embeddings on create/update

### Phase 4: Similar Search (Week 4-5)
- [ ] Implement similar search endpoint
- [ ] Add "Find similar" UI component
- [ ] Test similarity search quality
- [ ] Tune vector index parameters

### Phase 5: Hybrid Search (Week 5-6)
- [ ] Implement hybrid ranking algorithm
- [ ] Add search mode selector
- [ ] Tune weight parameters
- [ ] Performance optimization
- [ ] User testing and refinement

## Technical Considerations

### Performance
- **FTS5 indexes**: Fast, but larger storage footprint
- **Vector indexes**: Use DiskANN for approximate nearest neighbor (fast at scale)
- **Query optimization**: Limit results, use pagination
- **Caching**: Cache frequent searches on frontend

### Storage
- **FTS5**: ~30-50% overhead on indexed text
- **Embeddings**: 384 dims × 4 bytes = 1.5KB per resource
- **Total estimate**: ~2-3KB per searchable resource

### Extensibility
- **New resource types**: Add FTS5 table + triggers + embedding column
- **Search plugins**: Abstract search interface for custom search logic
- **Ranking customization**: Configurable weights per resource type

### Privacy & Offline
- **Local-first**: All search works offline
- **Embeddings**: Use local model for privacy, or optional cloud API
- **Sync**: Search indexes sync with remote database

## API Design

### Search Request
```typescript
interface SearchRequest {
  q: string;                    // Search query
  types?: ResourceType[];        // Filter by resource types
  tags?: string[];              // Filter by tag IDs
  mode?: 'fuzzy' | 'similar' | 'hybrid';
  limit?: number;               // Default: 50
  offset?: number;              // For pagination
  dateFrom?: string;            // ISO date
  dateTo?: string;              // ISO date
}
```

### Search Response
```typescript
interface SearchResponse {
  results: SearchResult[];
  total: number;
  query: string;
  mode: 'fuzzy' | 'similar' | 'hybrid';
}

interface SearchResult {
  type: ResourceType;
  id: string;
  resource: Entry | Task | SubTask | Goal | Tag;
  score: number;                // Relevance score
  highlights?: string[];         // Matched text snippets
  matchedFields?: string[];     // Which fields matched
}
```

## Future Enhancements

1. **Search Analytics**: Track popular searches, zero-result queries
2. **Search Suggestions**: Autocomplete based on history
3. **Saved Searches**: Save complex search queries
4. **Search Filters UI**: Advanced filter panel
5. **Multi-language Support**: Language-specific tokenization
6. **Search Index Optimization**: Periodic optimization/merging
7. **Search Result Export**: Export search results

## Questions to Consider

1. **Embedding Model**: Local (privacy) vs API (simplicity)?
2. **Embedding Dimensions**: 384 (faster) vs 1536 (better quality)?
3. **Search Default**: Fuzzy-first or hybrid by default?
4. **Index Maintenance**: Auto-optimize or manual?
5. **Search Scope**: Include deleted/archived items?

## Next Steps

1. Review and approve this plan
2. Decide on embedding approach (local vs API)
3. Start Phase 1 implementation
4. Set up testing with sample data
5. Iterate based on user feedback
