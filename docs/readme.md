# Aether Docs

This directory is the home for human-facing project documentation.

## Start Here

- [Project README](./reference/project-readme.md): repo overview and local development entrypoint.
- [Architecture](./reference/architecture.md): system overview, directory map, and backend/frontend gaps.
- [Flows](./reference/flows.md): Mermaid diagrams for app runtime, onboarding, CRUD, transcription, sync, updater, and v1 surface.
- [Completed Work](./milestones/completed-work.md): decisions and project-shaping work already done.
- [V1 Release Checklist](./milestones/v1-release-checklist.md): the current release checklist.
- [Features Planned But Not Implemented](./planned/features-not-implemented.md): planned or backend-backed features that are not ready in the product UI.
- [AI Journal Enrichment](./planned/ai-journal-enrichment.md): planned AI enrichment for journal insights, weekly summaries, and relation suggestions.

## Directory Layout

- `reference/`: durable architecture, product, sync, and package reference docs.
- `milestones/`: release and milestone planning docs that track work already scoped or completed.
- `planned/`: feature ideas and planned work that should not be confused with shipped behavior.

## V1 Direction

V1 should be smaller but sealed. The release surface is journal, tasks, goals, settings, sync, updater, and stable audio transcription inside the journal. Canvas is out for v1. Bookmarks and global search should either be finished enough to feel real or hidden.
