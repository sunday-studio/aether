---
name: Tauri Commands Migration
overview: Migrate from HTTP REST server to Tauri commands while maintaining OpenAPI spec generation, TypeScript types, and TanStack Query compatibility with optimistic updates.
todos:
  - id: build-time-openapi
    content: Create build script to generate OpenAPI spec at build time and write to backend/docs/swagger.json
    status: completed
  - id: convert-handlers
    content: Convert all Axum handlers to Tauri commands, maintaining same request/response types
    status: completed
    dependencies:
      - build-time-openapi
  - id: tauri-state
    content: Set up Tauri state management for database access in commands, initialize DB on app start
    status: completed
  - id: custom-fetch
    content: Create custom fetch adapter that maps HTTP routes to Tauri command invocations
    status: completed
    dependencies:
      - convert-handlers
  - id: remove-http
    content: Remove all HTTP server dependencies (axum, tower, tower-http) and server startup code
    status: completed
    dependencies:
      - custom-fetch
  - id: update-openapi-annotations
    content: Update utoipa annotations in command files to maintain OpenAPI spec generation
    status: completed
    dependencies:
      - convert-handlers
---

# Migration from HTTP REST to Tauri Commands

## Overview

This plan migrates the Rust backend from an HTTP REST server (Axum) to Tauri commands, while preserving:

- OpenAPI spec generation for type safety
- Automated TypeScript type generation via Orval
- TanStack Query hooks with optimistic updates
- Existing API structure and response types

## Architecture Changes

### Current Flow

```
Frontend (React) → ky HTTP client → Axum HTTP Server → Database
                     ↓
              Orval-generated hooks
                     ↓
              OpenAPI spec (runtime)
```

### New Flow

```
Frontend (React) → Custom fetch adapter → Tauri Commands → Database
                     ↓
              Orval-generated hooks (unchanged)
                     ↓
              OpenAPI spec (build-time)
```

## Implementation Plan

### Phase 1: OpenAPI Spec Generation (Build-Time)

**Files to modify:**

- `desktop/src-tauri/backend/src/api/openapi.rs` - Add build-time spec export
- `desktop/src-tauri/backend/build.rs` - Generate spec during build
- `desktop/src-tauri/backend/Cargo.toml` - Add build script dependencies

**Changes:**

1. Create a build script that generates OpenAPI JSON from `ApiDoc` struct
2. Write the spec to `backend/docs/swagger.json` during build
3. Remove runtime HTTP endpoint for OpenAPI spec
4. Update Orval config to use the build-time generated spec

### Phase 2: Convert Handlers to Tauri Commands

**Files to modify:**

- `desktop/src-tauri/backend/src/handlers/*.rs` - Convert all handlers
- `desktop/src-tauri/src/lib.rs` - Register Tauri commands
- `desktop/src-tauri/backend/src/lib.rs` - Remove HTTP server startup
- `desktop/src-tauri/backend/src/api/routes.rs` - Remove (no longer needed)

**Pattern for conversion:**

- Convert `async fn handler(State(state): State<DbState>, ...)` 
- To `#[tauri::command] async fn handler(app: tauri::AppHandle, ...)`
- Use Tauri's state management for database access
- Maintain same request/response types

**Key changes:**

- Replace `State<DbState>` with Tauri's state management
- Replace `Path`, `Json` extractors with direct function parameters
- Replace `IntoResponse` with direct return types
- Remove HTTP status codes (use Result types instead)

### Phase 3: Create Tauri Command Bridge

**New files:**

- `desktop/src-tauri/backend/src/commands/mod.rs` - Command module
- `desktop/src-tauri/backend/src/commands/tags.rs` - Tag commands
- `desktop/src-tauri/backend/src/commands/entries.rs` - Entry commands
- `desktop/src-tauri/backend/src/commands/tasks.rs` - Task commands
- `desktop/src-tauri/backend/src/commands/goals.rs` - Goal commands
- `desktop/src-tauri/backend/src/commands/trash.rs` - Trash commands
- `desktop/src-tauri/backend/src/commands/sync.rs` - Sync commands

**Structure:**

- Each command file mirrors the handler structure
- Commands use `#[tauri::command]` macro
- Database state managed via Tauri's state management

### Phase 4: Custom Fetch Adapter

**Files to modify:**

- `desktop/src/lib/api-client.ts` - Replace HTTP calls with Tauri commands
- `desktop/orval.config.ts` - Keep same config (works with custom mutator)

**Implementation:**

- Map HTTP routes to Tauri command names
- Convert HTTP methods and paths to command invocations
- Maintain same response format `{ data, status, headers }`
- Handle errors consistently with HTTP-like status codes

**Route mapping example:**

- `GET /v1/tags` → `invoke('get_all_tags')`
- `POST /v1/tags` → `invoke('create_tag', { data })`
- `GET /v1/entry/:id` → `invoke('get_entry_by_id', { id })`

### Phase 5: Remove HTTP Dependencies

**Files to modify:**

- `desktop/src-tauri/backend/Cargo.toml` - Remove axum, tower, tower-http
- `desktop/src-tauri/Cargo.toml` - Remove tauri-plugin-http
- `desktop/src-tauri/tauri.conf.json` - Remove HTTP plugin config
- `desktop/package.json` - Remove ky dependency (optional, keep for now)
- `desktop/src-tauri/src/lib.rs` - Remove HTTP server startup

**Cleanup:**

- Remove all Axum route definitions
- Remove HTTP server initialization
- Remove CORS configuration
- Keep utoipa for OpenAPI generation only

### Phase 6: Database State Management

**Files to modify:**

- `desktop/src-tauri/backend/src/db/db.go` - Add Tauri state management
- `desktop/src-tauri/src/lib.rs` - Initialize database state

**Changes:**

- Use Tauri's `manage()` to store database state
- Initialize database during Tauri app startup
- Run migrations on app start
- Remove background HTTP server thread

### Phase 7: Update OpenAPI Annotations

**Files to modify:**

- `desktop/src-tauri/backend/src/handlers/*.rs` → `commands/*.rs`

**Changes:**

- Keep all `#[utoipa::path]` annotations
- Update path annotations to reflect command structure (for documentation)
- Maintain all schema definitions
- Ensure spec generation still works

## Key Design Decisions

1. **Keep OpenAPI Spec**: Generate at build-time, not runtime
2. **Maintain API Compatibility**: Same request/response types
3. **Preserve TanStack Query**: Custom fetch adapter maintains compatibility
4. **Optimistic Updates**: Work unchanged since hooks remain the same
5. **Type Safety**: Orval continues to generate types from OpenAPI spec

## Migration Strategy

1. **Incremental**: Convert one handler module at a time
2. **Test**: Verify each module works before moving to next
3. **Backward Compatible**: Keep HTTP server running during transition (optional)
4. **Cleanup**: Remove HTTP dependencies only after all commands work

## Files Summary

**New Files:**

- `desktop/src-tauri/backend/src/commands/mod.rs`
- `desktop/src-tauri/backend/src/commands/*.rs` (7 files)
- `desktop/src-tauri/backend/build.rs`

**Modified Files:**

- `desktop/src-tauri/backend/src/lib.rs`
- `desktop/src-tauri/backend/src/api/openapi.rs`
- `desktop/src-tauri/backend/Cargo.toml`
- `desktop/src-tauri/src/lib.rs`
- `desktop/src/lib/api-client.ts`
- All handler files (converted to commands)

**Removed Files:**

- `desktop/src-tauri/backend/src/api/routes.rs`
- `desktop/src-tauri/backend/src/main.rs` (or repurpose for spec generation)

## Testing Checklist

- [ ] OpenAPI spec generates correctly at build time
- [ ] All Tauri commands register and work
- [ ] Custom fetch adapter routes correctly
- [ ] TanStack Query hooks work unchanged
- [ ] Optimistic updates function properly
- [ ] TypeScript types generate correctly
- [ ] Database operations work
- [ ] Error handling works
- [ ] No HTTP dependencies remain