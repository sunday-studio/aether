# Completed Work

This document records project-shaping work that has already been done or decided, so it does not get lost while v1 is being narrowed.

## Sync Hardening

- Implemented the sync v2 direction for device enrollment and per-device authentication.
- Split server enrollment seed phrase from the sync passphrase concept.
- Required device token authentication across protected sync-server endpoints.
- Fixed pull pagination around deterministic cursors.
- Added push batch idempotency.
- Updated the desktop Rust sync client to participate in the new auth/cursor model.

## Documentation

- Created a repo-level `docs/` area for human-facing documentation.
- Added architecture and flow docs with Mermaid diagrams.
- Moved human-facing README, feature, and sync docs under `docs/reference/`.
- Added a v1 release checklist under `docs/milestones/`.
- Added planned-but-not-implemented feature tracking under `docs/planned/`.

## Tooling

- Switched frontend linting and formatting setup from Biome/Prettier to OXC.
- Added `oxlint` and `oxfmt` package scripts in `desktop/package.json`.
- Added VS Code/Cursor settings and extension recommendations for OXC.
- Added agent instructions in `AGENTS.md` for naming and commit message conventions.

## Product Direction Decisions

- Canvas is out for v1.
- Updater is in scope for v1.
- First-run onboarding is in scope for v1.
- Local search model setup is visible for v1.
- Canvas, bookmarks, graph, journal audio/transcription, AI enrichment, provider-key setup, and broad search should stay out of v1.

## Updater Readiness

- Added a global update button that appears when a new signed update is available.
- Added updater download progress events from the Rust updater command.
- Persisted updater preferences and skipped versions in app config.
- Added release and updater testing steps under `docs/milestones/release-testing-plan.md`.

## V1 Surface Cleanup

- Kept bottom navigation, command palette, and global shortcuts scoped to Journal, Tasks, and Settings.
- Removed Canvas, Bookmarks, and Graph routes from the v1 surface.
- Removed editor resource-link autocomplete from the v1 surface.
- Removed journal audio/transcription and AI setup from the v1 surface.
