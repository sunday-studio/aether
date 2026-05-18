# Search And RAG Foundation

This is the implementation handoff for rebuilding Aether search as the foundation for related notes, weekly summaries, AI enrichment, and future RAG/chat.

## Goal

Build one reliable local retrieval layer:

- Search visible user text, not raw Lexical JSON.
- Return unified results for entries, tasks, goals, tags, and bookmarks.
- Keep indexes local and rebuildable.
- Use embeddings only after keyword search is solid.
- Let AI features retrieve clean context without knowing table internals.

## Phase 1: Clean Text Index

- [x] Add Rust Lexical-to-plain-text extraction for journal entries.
- [x] Add `search_documents` as the canonical local search index.
- [x] Store `resource_type`, `resource_id`, `chunk_index`, `title`, `text`, `text_hash`, `source_updated_at`, `created_at`, and `updated_at`.
- [x] Index entries from cleaned Lexical text.
- [x] Index tasks, goals, tags, and bookmarks from their title/name/description fields.
- [x] Do not sync `search_documents`.

## Phase 2: Indexing Service

- [x] Add a backend search indexer module.
- [ ] Add `reindex_resource(resource_type, resource_id)`.
- [x] Add `reindex_search()`.
- [x] Add `get_search_index_status()`.
- [ ] Call resource reindexing after create/update/delete for indexed resources.
- [ ] Ensure deleted resources are removed from search.

## Phase 3: Keyword Search

- [ ] Build FTS over `search_documents`.
- [ ] Replace raw per-table search with normalized search results.
- [ ] Support filters: `q`, `types`, `tags`, `date_from`, `date_to`, `limit`, `cursor`.
- [ ] Return `resourceType`, `resourceId`, `title`, `preview`, `score`, `matchKind`, `highlights`, and timestamps.
- [ ] Make `mode=keyword` the default.

## Phase 4: Embedding Storage

- [ ] Add `search_embeddings` tied to `search_documents`.
- [ ] Store model name, dimensions, vector, `text_hash`, and timestamps.
- [ ] Do not sync embeddings.
- [ ] Rebuild embeddings per device from synced source data.

## Phase 5: Real Local Embeddings

- [ ] Replace placeholder hash embeddings with a real local embedding provider.
- [ ] Add `index_embeddings()`.
- [ ] Add `index_resource_embedding(resource_type, resource_id)`.
- [ ] Add `get_embedding_status()`.
- [ ] Skip semantic search gracefully until embeddings are available.

## Phase 6: Hybrid Search

- [ ] Implement `mode=semantic`.
- [ ] Implement `mode=hybrid`.
- [ ] Merge keyword and semantic results by resource id.
- [ ] Start with simple scoring: keyword weight higher than semantic.
- [ ] Add small boosts for exact title/name matches, tags, pinned entries, incomplete tasks, and current goals.

## Phase 7: RAG Retrieval APIs

- [ ] Add `find_related_resources(resource_type, resource_id, limit)`.
- [ ] Add `retrieve_context(query, filters)`.
- [ ] Add `retrieve_week_context(start_date, end_date)`.
- [ ] Return source ids, clean excerpts, dates, resource types, and scores.
- [ ] Use these APIs for AI enrichment and weekly summaries later.

## Phase 8: Product Search UI

- [ ] Add real command-palette/global search.
- [ ] Group or label results by resource type.
- [ ] Show useful titles and previews.
- [ ] Open the selected resource from each result.
- [ ] Keep search usable offline.

## Acceptance Checklist

- [ ] Journal search matches visible text, not JSON internals.
- [ ] Entries, tasks, goals, tags, and bookmarks return through one result shape.
- [ ] Updated resources update the search index.
- [ ] Deleted resources never appear.
- [ ] Search works offline.
- [ ] Embeddings are optional and rebuildable.
- [ ] Search indexes, FTS tables, embedding rows, and jobs are not synced.
- [ ] AI/RAG code can retrieve context through retrieval APIs without direct table-specific queries.
