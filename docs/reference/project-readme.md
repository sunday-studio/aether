# Aether

Aether is a local-first desktop journal and task-management app built with Tauri, Rust, React, and TypeScript.

The repo contains two main parts:

- `desktop/` - the Tauri desktop app
- `sync-server/` - an optional end-to-end encrypted sync server

## V1 Product Surface

V1 keeps:

- Journal writing and editing.
- Tasks, subtasks, goals, and goal instances.
- Activity tracking and the activity heatmap.
- Trash view, restore behavior, and soft-delete workflows.
- Image and video media blobs, including media sync compatibility.
- Local search over journal entries and tasks.
- Local search model setup.
- Settings for sync, search, updater, and diagnostics where present.
- Self-hosted encrypted sync and sync-server compatibility.
- OTA updater.

V1 does not ship canvas, bookmarks, graph, journal audio recording, transcription, AI journal enrichment, provider-key setup, transcription model management, resource-link APIs, bookmark metadata extraction, or broad global search result types.

Existing database migrations are preserved for installed-user compatibility. A table that still exists because of a migration is not, by itself, a shipped feature.

## What You Need

For local development:

- `pnpm` for the desktop frontend/tooling
- `Rust` and `cargo`
- Tauri system prerequisites for your OS
- `Docker` if you want to run the sync server in a container instead of with Cargo

## Repository Layout

### Top Level

- `desktop/` - the main desktop application
- `sync-server/` - the standalone sync backend
- `docs/reference/features.md` - current v1 feature inventory
- `docs/reference/sync.md` - sync architecture and behavior notes
- `.github/` - GitHub workflow and PR metadata

### `desktop/`

- `src/` - React app code
- `src/components/` - shared UI and editor components
- `src/context/` - React context providers
- `src/features/` - v1 feature areas such as journal, tasks, settings, and onboarding
- `src/hooks/` - reusable frontend hooks
- `src/lib/` - API client and other shared frontend utilities
- `src/store/` - frontend state helpers
- `src/styles/` - global theme and color styles
- `src/openapi/` - OpenAPI spec used to generate the frontend SDK
- `src/aether-sdk/` - generated TypeScript client code
- `public/` - static assets and fonts
- `src-tauri/` - Rust backend for the desktop app
- `src-tauri/src/` - Tauri commands, database code, sync engine, media, search, settings, and utilities
- `src-tauri/migrations/` - local database schema migrations
- `src-tauri/tests/` - Rust integration tests
- `orval.config.ts` - SDK generation config for `src/aether-sdk/`
- `vite.config.ts` - Vite config for the desktop frontend

### `sync-server/`

- `src/main.rs` - server entrypoint
- `src/handlers.rs` - HTTP and WebSocket route handlers
- `src/storage.rs` - SQLite and blob storage logic
- `src/models.rs` - request and response models
- `src/lib.rs` - shared server wiring
- `data/` - local runtime data directory for `sync.db` and synced media blobs
- `Dockerfile` - container build for the sync server
- `docker-compose.yml` - local container run config
- `docker-compose.example.yml` - example with `SERVER_PASSPHRASE`

## Run Locally

### 1. Run the sync server (optional)

You only need this if you want to test sync.

```bash
cd sync-server
DATA_ROOT=./data cargo run
```

The server listens on `http://localhost:8080`.

### 2. Run the desktop app

```bash
cd desktop
pnpm install
pnpm run tauri:dev
```

Useful desktop commands:

```bash
cd desktop
pnpm run dev
pnpm run build
pnpm run generate:sdk
```

## Local Sync Setup

If the sync server is running:

1. Open the desktop app.
2. Go to `Settings -> Sync`.
3. Enter `http://localhost:8080` as the server URL.
4. Enter the server seed phrase.
5. Enter a sync passphrase with at least 12 characters.
6. Save and run sync.

More detail on sync behavior lives in [`sync.md`](./sync.md).

## Notes For Contributors

- The desktop app stores local database and media state under platform application-support directories.
- `desktop/src/aether-sdk/` is generated code. If you change the OpenAPI spec, regenerate it.
- Do not edit, remove, or rename existing files under `desktop/src-tauri/migrations/` for v1 surface cleanup.
