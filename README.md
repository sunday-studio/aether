# Aether

Aether is a local-first desktop knowledge and productivity app built with Tauri, Rust, React, and TypeScript.

The repo contains two main parts:

- `desktop/` - the Tauri desktop app
- `sync-server/` - an optional end-to-end encrypted sync server

<table>
  <tr>
    <td><img src="https://assets.casprine.com/personal-space/Screenshot%202026-05-03%20at%2010.19.59.png" alt="Aether screenshot 1" /></td>
    <td><img src="https://assets.casprine.com/personal-space/Screenshot%202026-05-03%20at%2010.57.44.png" alt="Aether screenshot 2" /></td>
    <td><img src="https://assets.casprine.com/personal-space/Screenshot%202026-05-03%20at%2011.21.36.png" alt="Aether screenshot 3" /></td>
  </tr>
</table>

## What You Need

For local development:

- `bun` for the desktop frontend/tooling
- `Rust` and `cargo`
- Tauri system prerequisites for your OS
- `Docker` if you want to run the sync server in a container instead of with Cargo

## Repository Layout

### Top level

- `desktop/` - the main desktop application
- `sync-server/` - the standalone sync backend
- `FEATURES.md` - product and feature inventory
- `SYNC.md` - sync architecture and behavior notes
- `.github/` - GitHub workflow and PR metadata

### `desktop/`

- `src/` - React app code
- `src/components/` - shared UI and editor components
- `src/context/` - React context providers
- `src/features/` - feature areas such as journal, tasks, graph, canvas, bookmarks, and settings
- `src/hooks/` - reusable frontend hooks
- `src/lib/` - API client and other shared frontend utilities
- `src/store/` - frontend state helpers
- `src/styles/` - global theme and color styles
- `src/openapi/` - OpenAPI spec used to generate the frontend SDK
- `src/aether-sdk/` - generated TypeScript client code
- `public/` - static assets and fonts
- `src-tauri/` - Rust backend for the desktop app
- `src-tauri/src/` - Tauri commands, database code, sync engine, media, transcription, settings, and utilities
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

#### Option A: Cargo

```bash
cd sync-server
DATA_ROOT=./data cargo run
```

The server listens on `http://localhost:8080`.

Optional environment variables:

- `DATA_ROOT` - where the server stores `sync.db` and blob files. Defaults to `./data`.
- `SERVER_PASSPHRASE` - optional server-side registration passphrase.

#### Option B: Docker

```bash
cd sync-server
docker compose up --build
```

This mounts `sync-server/data/` into the container so data persists locally.

### 2. Run the desktop app

```bash
cd desktop
bun install
bun run tauri:dev
```

`tauri:dev` starts the Vite dev server and the Tauri shell together. The frontend dev server runs at `http://localhost:1420` under the Tauri dev workflow.

Useful desktop commands:

```bash
cd desktop
bun run dev
bun run build
bun run generate:sdk
```

- `bun run dev` - run the frontend only
- `bun run build` - production frontend build
- `bun run generate:sdk` - regenerate `src/aether-sdk/` from `src/openapi/spec.json`

## Local Sync Setup

If the sync server is running:

1. Open the desktop app.
2. Go to `Settings -> Sync`.
3. Enter `http://localhost:8080` as the server URL.
4. Enter a passphrase with at least 12 characters.
5. Save and run sync.

More detail on sync behavior lives in [`SYNC.md`](./SYNC.md).

## Notes For Contributors

- The desktop app stores its own local database and media state under `desktop/src-tauri/` during development.
- `desktop/src/aether-sdk/` is generated code. If you change the OpenAPI spec, regenerate it.
- The checked-in sub-readmes in `desktop/` and `sync-server/` are minimal; this root README is the main starting point for local development.
