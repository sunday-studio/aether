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
- AI key setup belongs in onboarding, using the existing encrypted settings path.
- Bookmarks and global search should be finished or hidden for v1.

