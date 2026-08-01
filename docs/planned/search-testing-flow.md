# Search Testing Flow

This document tracks how agents should verify the v1 search scope.

## Static Verification

- Run `cargo check` from `desktop/src-tauri`.
- Run `cargo test search_document --lib` from `desktop/src-tauri`.
- Regenerate OpenAPI with `make generate-openapi`.
- Regenerate the frontend SDK with `make generate-sdk`.
- Confirm API routes are present in `desktop/src/openapi/spec.json`.
- Confirm desktop REST-style routes are mapped in `desktop/src/lib/api-client.ts`.

## Repository Tests

- Verify Lexical text extraction ignores JSON structure and returns visible text.
- Verify full reindex covers entries and task-management resources.
- Verify single-resource reindex accepts `entry`, `task`, `subtask`, and `goal`.
- Verify single-resource reindex rejects non-v1 resource types.
- Verify create/update/delete paths refresh or remove derived search documents.
- Verify deleted entries and task-management resources are removed from `search_documents`.
- Verify invalid Lexical JSON does not break full reindex.
- Verify search index counts match seeded entries, tasks, subtasks, and goals.
- Verify keyword-search tests cover normalized results from `search_documents`.
- Verify filters for `types`, `tags`, `date_from`, `date_to`, `limit`, and `offset`.
- Verify cursor pagination for keyword search.
- Verify tag filters apply only to entries and task-management resources.
- Verify semantic and hybrid search over `search_embeddings`.
- Verify ranking boosts for title, tags, pinned entries, and incomplete tasks.

## Runtime/In-App Verification Path

Use this path before release:

- Start the desktop app.
- Create or confirm at least one journal entry, task, subtask, and goal with unique searchable words.
- Trigger `POST /v1/search/index/reindex` through the frontend API client.
- Confirm `GET /v1/search/index/status` reports non-zero entry, task, subtask, and goal counts.
- Call `GET /v1/search?q=<word>&mode=keyword&limit=1` through the frontend API client.
- Confirm the response includes `results`, `nextCursor`, `hasMore`, `resourceType`, `resourceId`, `title`, `preview`, `score`, and `matchKind`.
- Call the same search with `cursor=<nextCursor>` when `hasMore` is true and confirm the next page does not repeat the first result.
- Call `GET /v1/search?q=<word>&tags=<tag-id>` and confirm untagged entry/task resources are excluded.
- Rebuild embeddings from Settings.
- Call `GET /v1/search?q=<word>&mode=semantic` and confirm results return `matchKind=semantic`.
- Call `GET /v1/search?q=<word>&mode=hybrid` and confirm results return `matchKind=hybrid`.

## Local Model Smoke Test

Unit tests use deterministic fallback embeddings so they do not download the local model in CI.

- Start the desktop app with `npm run dev` from `desktop/`.
- Download the `all-MiniLM-L6-v2` search embedding model from onboarding or Settings.
- Create one journal entry and task-management item with different but related natural-language phrasing.
- Rebuild the search document index and search embeddings.
- Confirm `GET /v1/search?q=<related phrase>&mode=semantic` returns journal and task-management results with `matchKind=semantic`.
- Confirm `GET /v1/search?q=<keyword>&mode=hybrid` returns journal and task-management results with `matchKind=hybrid`.
- Confirm default search responses do not include bookmark, canvas, or graph resources.
