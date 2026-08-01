# Aether Docs

This directory is the home for human-facing project documentation.

## Start Here

- [Project README](./reference/project-readme.md): repo overview and local development entrypoint.
- [Architecture](./reference/architecture.md): current v1 system overview and directory map.
- [Features](./reference/features.md): v1 feature inventory.
- [Flows](./reference/flows.md): Mermaid diagrams for app runtime, onboarding, CRUD, search, sync, updater, and v1 surface.
- [Diagnostics](./reference/diagnostics.md): local debug log export behavior and telemetry recommendation.
- [Completed Work](./milestones/completed-work.md): decisions and project-shaping work already done.
- [V1 Release Checklist](./milestones/v1-release-checklist.md): the current release checklist.
- [Release Testing Plan](./milestones/release-testing-plan.md): checks for release readiness, smoke testing, and updater validation.
- [Features Planned But Not Implemented](./planned/features-not-implemented.md): post-v1 candidates that are not in the v1 product surface.

## Directory Layout

- `reference/`: durable architecture, product, sync, and package reference docs.
- `milestones/`: release and milestone planning docs that track scoped or completed work.
- `planned/`: future feature ideas that should not be confused with shipped behavior.

## V1 Direction

V1 is intentionally narrow. The release surface is journal writing, tasks, goals, activity tracking, trash, image/video media blobs, local journal/task search, local search model setup, settings, encrypted sync, diagnostics where present, and the OTA updater.

Canvas, bookmarks, graph, journal audio/transcription, AI journal enrichment, provider-key setup, transcription model management, resource-link APIs, bookmark metadata extraction, and broad global search are not v1 product surfaces.
