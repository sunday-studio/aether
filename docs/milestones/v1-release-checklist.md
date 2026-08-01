# V1 Release Checklist

This is the working list to run before cutting v1. The intent is to ship a smaller, coherent app with no placeholder surfaces.

## 1. Lock V1 Product Surface

- Keep journal writing/editor.
- Keep tasks, subtasks, goals, and goal instances.
- Keep activity tracking and the activity heatmap.
- Keep trash view, restore behavior, and repositories.
- Keep image and video media blobs.
- Keep local journal/task search.
- Keep local search model setup.
- Keep self-hosted encrypted sync and sync-server compatibility.
- Keep updater.
- Keep settings required for sync, search, updater, and diagnostics where present.
- Keep all existing database migrations unchanged.

## 2. Exclude Non-V1 Surfaces

- Remove Canvas routes, commands, repositories, API exposure, and unused canvas dependencies.
- Remove Bookmarks routes, commands, repositories, metadata extraction, API exposure, and bookmark-specific search indexing.
- Remove Graph routes and graph-only backend/link API exposure.
- Remove journal audio recording, audio commands/modules, transcription commands/modules, provider setup, transcription settings, and transcription copy.
- Remove AI journal enrichment commands, repositories, modules, settings copy, and docs that imply it ships in v1.
- Keep command palette search scoped to journal entries and tasks.
- Keep goals available for task-management navigation without broadening search result types.

## 3. Onboarding And Settings

- Collect basic user profile data.
- Offer optional sync setup.
- Offer optional local search model setup.
- Persist onboarding completion in settings.
- Reduce the old AI settings section to local search model setup.
- Remove provider-key and transcription model-management settings.

## 4. Updater

- Keep Settings > What's New.
- Keep the global update indicator wired to updater state.
- Verify manual update checks.
- Verify update available state.
- Verify skip version.
- Verify download and install.
- Verify update preferences persist.
- Verify auto-check behavior if enabled.
- Run the updater section of the [release testing plan](./release-testing-plan.md).

## 5. Sync

- Verify setup with server URL, server seed phrase, and sync passphrase.
- Verify reconnect with sync passphrase.
- Verify manual sync.
- Verify periodic or websocket-triggered sync if enabled.
- Verify image/video media sync policy: auto and on-demand.
- Keep sync failure messages understandable and non-debuggy.
- Preserve compatibility with the sync server and historical encrypted changes.

## 6. Navigation And App Shell Cleanup

- Ensure bottom navigation exposes only v1 features.
- Ensure command palette search only opens journal entries and tasks.
- Remove routes that land on placeholder or stripped screens.
- Remove keyboard shortcuts for stripped features.
- Remove stale generated SDK hooks for stripped commands.

## 7. Verification

- Run `git diff --check`.
- Run `pnpm run lint` from `desktop/`.
- Run `pnpm run build` from `desktop/`.
- Run `cargo fmt --check` from `desktop/src-tauri/`.
- Run `cargo check` from `desktop/src-tauri/`.
- Run `cargo test` from `desktop/src-tauri/`.
- Run targeted stale-reference searches for stripped routes, commands, dependencies, and generated OpenAPI output.
- Smoke first-launch onboarding.
- Smoke journal entry creation.
- Smoke task and goal creation.
- Smoke local search model setup and journal/task search.
- Smoke sync configure, sync now, reconnect, and media policy.
- Smoke updater settings and manual check.
