# Search Testing Flow

This document tracks how agents should verify the Search And RAG Foundation work.

## Current Verification

- [x] Run `cargo check` from `desktop/src-tauri`.
- [x] Run `cargo test search_text --lib` from `desktop/src-tauri`.
- [x] Regenerate OpenAPI with `cargo run --manifest-path desktop/src-tauri/tools/Cargo.toml`.
- [x] Confirm API routes are present in `desktop/src/openapi/spec.json`.
- [x] Confirm desktop REST-style routes are mapped in `desktop/src/lib/api-client.ts`.

## Phase 1 And 2 Test Flow

- [x] Verify Lexical text extraction ignores JSON structure and returns visible text.
- [x] Verify `search_documents` migration exists and has no sync columns/triggers.
- [x] Verify full reindex command exists.
- [x] Verify single-resource reindex command exists.
- [x] Verify create/update/delete paths refresh or remove derived search documents.

## Next To Test

- [x] Add backend integration tests for `SearchDocumentRepository::reindex_all`.
- [x] Add backend integration tests for `SearchDocumentRepository::reindex_resource`.
- [x] Verify deleted tasks are removed from `search_documents`.
- [x] Verify invalid Lexical JSON does not break full reindex.
- [x] Verify search index counts match seeded entries, tasks, goals, tags, and bookmarks.
- [x] Add Phase 3 keyword-search tests for normalized results from `search_documents`.
- [x] Add tests for supported Phase 3 filters: `types`, `date_from`, `date_to`, `limit`, and `offset`.
- [x] Add cursor pagination tests for keyword search.
- [x] Extend deleted-resource tests to entries, goals, and bookmarks.
- [x] Add coverage for `SearchDocumentRepository::delete_resource`.
- [x] Add search command tests for `semantic` and `hybrid` unavailable-mode errors.
- [x] Add tag-filter tests once tag filtering is implemented.
- [x] Add a runtime/in-app API verification path for Tauri commands before UI work.

## Curl Note

The desktop app currently exposes REST-shaped routes through the frontend Tauri API client, not a localhost HTTP server. Plain `curl` can verify the sync server, but it cannot directly call desktop commands until a local HTTP harness or dev-only command bridge exists.

When UI/runtime work starts, verify these routes through the app layer:

- `POST /v1/search/index/reindex`
- `POST /v1/search/index/resource`
- `GET /v1/search/index/status`
- `GET /v1/search`

## Runtime/In-App Verification Path

Use this path before building the search UI:

- Start the desktop app with `npm run dev` from `desktop/`.
- Create or confirm at least one journal entry, task, goal, tag, and bookmark with unique searchable words.
- Trigger `POST /v1/search/index/reindex` through the frontend API client.
- Confirm `GET /v1/search/index/status` reports non-zero counts for indexed resource types.
- Call `GET /v1/search?q=<word>&mode=keyword&limit=1` through the frontend API client.
- Confirm the response includes `results`, `nextCursor`, `hasMore`, `resourceType`, `resourceId`, `title`, `preview`, `score`, and `matchKind`.
- Call the same search with `cursor=<nextCursor>` when `hasMore` is true and confirm the next page does not repeat the first result.
- Call `GET /v1/search?q=<word>&tags=<tag-id>` and confirm untagged resources are excluded.
- Call `GET /v1/search?q=<word>&mode=semantic` and `mode=hybrid`; both should return unavailable-mode `400` errors until embeddings are indexed.

## Phase 4 Embedding Storage Test Flow

- [x] Verify `search_embeddings` is local derived storage with no sync columns or triggers.
- [x] Verify embeddings are scoped by `search_document_id` and `model_name`.
- [x] Verify vector byte storage round trips to `Vec<f32>`.
- [x] Verify dimension mismatches are rejected.
- [x] Verify deleting a `search_documents` row removes related embeddings.

## Phase 5 Embedding Indexing Test Flow

- [x] Verify full embedding indexing generates local vectors for `search_documents`.
- [x] Verify resource embedding indexing clears embeddings when the search document is missing.
- [x] Verify semantic and hybrid search still fail gracefully until real embedding search is implemented.
- [ ] Verify a real local model provider once an inference runtime is selected.

## Phase 7 Retrieval API Test Flow

- [x] Verify related-resource retrieval excludes the source resource.
- [x] Verify date-range context returns clean previews and source metadata.
- [x] Verify retrieval API routes are included in the frontend API client and OpenAPI spec.
