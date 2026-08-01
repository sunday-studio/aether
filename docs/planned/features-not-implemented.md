# Features Planned But Not Implemented

This document tracks planned or deferred work that should not be treated as shipped product behavior.

## Removed From V1 Surface

### Canvas

Canvas is not part of v1. Do not expose canvas routes, commands, repositories, API paths, generated SDK hooks, dependencies, navigation destinations, or keyboard shortcuts in the v1 branch.

### Bookmarks

Bookmarks are not part of v1. Do not expose bookmark routes, commands, repositories, metadata extraction, bookmark-specific tags/search indexing, API paths, generated SDK hooks, or navigation destinations in the v1 branch.

### Knowledge Graph

Graph is not part of v1. Do not expose graph routes or graph-only link APIs in the v1 branch.

### Journal Audio And Transcription

Journal audio recording and transcription are not part of v1. Do not expose audio commands, journal audio UI, transcription commands, transcription providers, transcription model management, provider validation, or transcription settings in the v1 branch.

### AI Journal Enrichment

AI journal enrichment is not part of v1. Do not expose entry insights, suggestions, weekly summaries, AI provider setup, provider-key copy, or AI enrichment APIs in the v1 branch.

### Broad Search And Relationship APIs

V1 search is scoped to journal entries and tasks. Related-resource retrieval, week context retrieval, bookmark search indexing, graph/link search, and non-v1 command palette result types are deferred.

## Later Candidate Features

These features can return after v1 only as complete product surfaces with deliberate routes, settings, commands, API exposure, tests, and docs:

- Canvas.
- Saved bookmarks.
- Knowledge graph.
- Journal audio recording.
- Transcription.
- AI journal enrichment.
- Full search results and semantic recall workflows.
- Relationship/resource-link APIs.

Existing database migrations and sync compatibility handlers may keep historical entity names so older installs and encrypted changes remain compatible. That compatibility does not make those entities v1 features.
