# Search Foundation

This document records the narrowed v1 search scope.

## V1 Goal

Build one reliable local retrieval layer for journal entries and tasks:

- Search visible user text, not raw Lexical JSON.
- Return unified results for entries and tasks only.
- Keep indexes local and rebuildable.
- Use embeddings only after keyword search is available.
- Keep semantic search optional based on local model setup.

## Implemented Scope

- Search documents are local derived rows.
- Journal entries are indexed from cleaned Lexical text.
- Tasks are indexed from title and description fields.
- Deleted entries and tasks are removed from search.
- Keyword, semantic, and hybrid search are supported.
- Embeddings are local and rebuildable per device.
- Command palette search opens only journal entries and tasks.
- Search index status reports total documents, entries, and tasks.

## Explicitly Out Of V1

- Goals as search results.
- Tags as search results.
- Bookmarks as search results.
- Related-resource retrieval APIs.
- Week-context retrieval APIs.
- AI/RAG retrieval APIs.
- Broad global search surfaces beyond journal entries and tasks.

Historical migrations may still contain broader search-support tables. The v1 command and product surface should stay scoped to entries and tasks.
