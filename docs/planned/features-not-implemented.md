# Features Planned But Not Implemented

This document tracks planned, deferred, or backend-backed work that should not be treated as shipped product behavior yet.

## V1 Deferred Or Hidden

### Canvas

Canvas has real frontend and backend implementation, including route code, React components, state, and canvas persistence. It is out for v1. Hide the route, keyboard shortcut, navigation paths, and resource-link destinations while leaving the implementation available for a later milestone.

### Bookmarks

The backend supports bookmark CRUD, metadata extraction, tagging, archive state, repository logic, and generated API hooks. The current frontend route is placeholder-level, so it is hidden for v1.

### Knowledge Graph

The backend supports graph/link retrieval and the frontend has a graph visualization route. The current experience is not polished enough for v1, so the route is hidden until labels, empty states, navigation, and resource-opening behavior are designed and implemented.

### Transcription Provider And Model Management

The backend can list providers, validate provider configuration, manage local Whisper models, save journal audio, and run transcription jobs. AI key setup stays visible for v1, but the visible journal audio/transcription UI is hidden and full provider/model management is not a complete settings surface.

### Sync Diagnostics

The backend exposes sync trigger check/test commands. These are development diagnostics, not user-facing v1 features. Keep them hidden unless a later support/debug mode is intentionally designed.

## Planned V1 Work Not Yet Implemented

### First-Run Onboarding

Add a first-launch onboarding gate before the main app. It should collect a display name, explain optional sync, validate sync setup when requested, offer optional AI key setup, and persist completion in settings.

### Navigation Cleanup

The v1 navigation should expose only finished surfaces. Canvas, Bookmarks, and Graph routes are hidden, placeholder destinations are not advertised, and command palette search should stay scoped to indexed, v1-ready resources.

### Backend-Frontend Gap Cleanup

For each backend capability, decide whether it is shipped, hidden, or deferred. The v1 app should not expose routes, settings, or controls that land on placeholder or debug-only behavior.

### AI Journal Enrichment

AI journal enrichment is planned as a local-first, editable suggestion layer for daily entry insights, weekly summaries, and relationship suggestions. See [AI Journal Enrichment](./ai-journal-enrichment.md) for the implementation direction. This should not be exposed as shipped behavior until the review/edit/dismiss flows, provider settings, sync behavior, and privacy copy are complete.

## Later Candidate Features

### Local Model Downloads

Local search model download is now part of onboarding and Settings. Local Whisper model download and verification exists at the backend level; a later product surface could let users manage local transcription models, but that should be designed as a full settings flow.

### Rich Bookmark Experience

A later milestone can make bookmarks a real saved-resource feature with list, filtering, metadata refresh, archive state, tags, and search integration.

### Search And Semantic Recall

Search now includes command palette results, local embeddings, and hybrid ranking. A later milestone can add a full search results view and deeper semantic recall workflows.

### Canvas Return

Canvas can return after v1 as a focused spatial-thinking surface, but it should come back as a complete user flow rather than a partially exposed route.

### Knowledge Graph Return

Graph can return after v1 as a discovery and relationship surface once it has a clear entry point, meaningful node labels, useful empty states, and resource navigation.
