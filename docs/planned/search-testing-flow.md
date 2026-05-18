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

- [ ] Add backend integration tests for `SearchDocumentRepository::reindex_all`.
- [ ] Add backend integration tests for `SearchDocumentRepository::reindex_resource`.
- [ ] Verify deleted entries, tasks, goals, and bookmarks are removed from `search_documents`.
- [ ] Verify invalid Lexical JSON does not break full reindex.
- [ ] Verify search index counts match seeded entries, tasks, goals, tags, and bookmarks.
- [ ] Add Phase 3 keyword-search tests for normalized results from `search_documents`.
- [ ] Add tests for supported Phase 3 filters: `types`, `date_from`, `date_to`, `limit`, and `offset`.
- [ ] Add tag-filter tests once tag filtering is implemented.
- [ ] Add a runtime/in-app API verification path for Tauri commands before UI work.

## Curl Note

The desktop app currently exposes REST-shaped routes through the frontend Tauri API client, not a localhost HTTP server. Plain `curl` can verify the sync server, but it cannot directly call desktop commands until a local HTTP harness or dev-only command bridge exists.

When UI/runtime work starts, verify these routes through the app layer:

- `POST /v1/search/index/reindex`
- `POST /v1/search/index/resource`
- `GET /v1/search/index/status`
- `GET /v1/search`
