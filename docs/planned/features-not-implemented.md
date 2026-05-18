# Features Planned But Not Implemented

This document tracks planned, deferred, or backend-backed work that should not be treated as shipped product behavior yet.

## V1 Deferred Or Hidden

### Canvas

Canvas has real frontend and backend implementation, including route code, React components, state, and canvas persistence. It is out for v1. Hide the route, keyboard shortcut, navigation paths, and resource-link destinations while leaving the implementation available for a later milestone.

### Bookmarks

The backend supports bookmark CRUD, metadata extraction, tagging, archive state, repository logic, and generated API hooks. The current frontend route is placeholder-level. For v1, either finish the bookmark list/create/edit/archive/tag flow or hide the route entirely.

### Global Search

The backend has fuzzy/hybrid search and linkable-resource search. Resource-link autocomplete uses backend search, but command-palette/global search is commented out. For v1, ship a complete global search experience or keep search hidden as an internal capability.

### Embeddings Management

The Rust backend exposes embedding model list, download, verify, and delete commands. There is no meaningful frontend management surface. Defer this until there is a clear user-facing AI/search feature that needs it.

### Transcription Provider And Model Management

The backend can list providers, validate provider configuration, and manage local Whisper models. The journal can use transcription, but full provider/model management is not a complete settings surface. For v1, expose only onboarding and journal transcription needs: API key entry, default provider choice, and provider validation.

### Sync Diagnostics

The backend exposes sync trigger check/test commands. These are development diagnostics, not user-facing v1 features. Keep them hidden unless a later support/debug mode is intentionally designed.

## Planned V1 Work Not Yet Implemented

### First-Run Onboarding

Add a first-launch onboarding gate before the main app. It should collect a display name, offer optional AI key setup, let the user choose a default transcription provider, validate credentials when requested, and persist completion in settings.

### Updater Notifications

The updater settings page exists, but the update event/toast listener path is commented out. Re-enable or finish update notifications so updater behavior is visible outside the settings screen.

### Navigation Cleanup

The v1 navigation should expose only finished surfaces. Remove or disable Canvas routes and shortcuts, hide placeholder destinations, and ensure the command palette does not advertise unfinished views.

### Backend-Frontend Gap Cleanup

For each backend capability, decide whether it is shipped, hidden, or deferred. The v1 app should not expose routes, settings, or controls that land on placeholder or debug-only behavior.

### AI Journal Enrichment

AI journal enrichment is planned as a local-first, editable suggestion layer for daily entry insights, weekly summaries, and relationship suggestions. See [AI Journal Enrichment](./ai-journal-enrichment.md) for the implementation direction. This should not be exposed as shipped behavior until the review/edit/dismiss flows, provider settings, sync behavior, and privacy copy are complete.

## Later Candidate Features

### Local Model Downloads

Local Whisper model download and verification exists at the backend level. A later product surface could let users manage local transcription models, but that should be designed as a full settings flow.

### Rich Bookmark Experience

A later milestone can make bookmarks a real saved-resource feature with list, filtering, metadata refresh, archive state, tags, and search integration.

### Search And Semantic Recall

Search can grow into a command palette, global results view, and semantic recall experience once embeddings and indexing behavior are productized.

### Canvas Return

Canvas can return after v1 as a focused spatial-thinking surface, but it should come back as a complete user flow rather than a partially exposed route.
