---
name: Remove LibSQL sync functionality
overview: Remove all sync and replication functionality from LibSQL while keeping the LibSQL dependency. This includes removing sync commands, handlers, UI components, and simplifying the database connection to local-only mode.
todos: []
---

# Remove LibSQL Sync Functionality

## Overview

Remove all sync and replication functionality from LibSQL while keeping the LibSQL dependency. The application will use LibSQL in local-only mode without any remote sync capabilities.

## Architecture Changes

### Current State

- Uses `libsql` crate with replication features enabled
- Supports sync with remote LibSQL servers via `configure_sync()`
- Has sync state management in `DbState` (sync_url, auth_token, has_sync_capability)
- Sync commands and handlers for configuring and triggering sync
- Sync UI in settings

### Target State

- Uses `libsql` crate in local-only mode only
- No sync/replication functionality
- Simplified `DbState` without sync fields
- No sync commands or handlers
- No sync UI

## Implementation Phases

### Phase 1: Simplify Database Connection

**File:** `desktop/src-tauri/src/db/connection.rs`

**Changes:**

1. Simplify `DbState` struct:
   ```rust
   pub struct DbState {
       pub database: Arc<Mutex<Arc<Database>>>,
   }
   ```


   - Remove: `sync_url: Arc<Mutex<Option<String>>>`
   - Remove: `auth_token: Arc<Mutex<Option<String>>>`
   - Remove: `has_sync_capability: Arc<Mutex<bool>>`

2. Update `initialize()` function:

   - Remove all sync-related comments and documentation
   - Keep only `Builder::new_local()` (already local-only)
   - Remove sync state initialization
   - Simplify to just create local database

3. Remove functions:

   - `configure_sync()` - entire function
   - `sync()` - entire function
   - `restore_sync_configuration()` - entire function
   - `get_sync_interval()` - helper function
   - Any other sync-related helper functions

4. Update `apply_sqlite_optimizations()`:

   - Remove any sync/replica-specific PRAGMA checks
   - Keep only local database optimizations

5. Update Cargo.toml dependency:

   - Change: `libsql = { version = "0.9", features = ["replication", "remote"] }`
   - To: `libsql = { version = "0.9" }` (remove replication and remote features)

### Phase 2: Remove Sync Commands

**Files:**

- `desktop/src-tauri/src/commands/sync.rs` - Delete entire file
- `desktop/src-tauri/src/commands/mod.rs` - Remove sync module export
- `desktop/src-tauri/src/lib.rs` - Remove sync command registrations

**Changes:**

1. Delete `sync.rs` file completely
2. Remove from `commands/mod.rs`:
   ```rust
   // Remove: pub mod sync;
   ```

3. Remove from `lib.rs` invoke_handler:
   ```rust
   // Remove:
   // sync::configure_sync,
   // sync::sync,
   ```


### Phase 3: Remove Sync Handlers

**Files:**

- `desktop/src-tauri/src/handlers/sync.rs` - Delete entire file
- `desktop/src-tauri/src/handlers/mod.rs` - Remove sync module export
- `desktop/src-tauri/src/api/openapi.rs` - Remove sync endpoints from OpenAPI spec

**Changes:**

1. Delete `handlers/sync.rs` file completely
2. Remove from `handlers/mod.rs`:
   ```rust
   // Remove: pub mod sync;
   ```

3. Remove sync endpoints from OpenAPI spec:

   - `POST /v1/sync/configure`
   - `POST /v1/sync`

### Phase 4: Update Frontend - Remove Sync UI

**SKIPPED** - Frontend changes are not part of this migration. Sync UI will remain but will not function (API endpoints removed).

### Phase 5: Clean Up Database Path and Documentation

**Files:**

- `desktop/src-tauri/src/db/connection.rs` - Update database path comments
- `remote-replica/` directory - Archive or add note that it's not used
- `FEATURES.md` - Remove sync features section

**Changes:**

1. Update database path in `connection.rs`:

   - Keep path as `./libsql-replica/local.db` (or rename to `./data/local.db`)
   - Update comments to clarify it's local-only

2. Update `remote-replica/README.md`:

   - Add note at top: "⚠️ Sync functionality has been removed. This directory is archived."
   - Or move to `archive/` directory

3. Update `FEATURES.md`:

   - Remove "6.1 Sync" section
   - Update technical features to note local-only database

### Phase 6: Remove Sync-Related Code Comments

**Files:** All files that mention sync

**Changes:**

- Remove comments referencing:
  - Turso Offline Sync
  - Remote replication
  - Multi-device sync
  - Sync intervals
  - Offline writes (in sync context)

### Phase 7: Update Error Handling

**File:** `desktop/src-tauri/src/error.rs`

**Changes:**

- Review if any sync-specific error variants exist
- Remove sync-related error cases if present

## Files to Modify

### Backend Files

1. **`desktop/src-tauri/Cargo.toml`**

   - Remove `replication` and `remote` features from libsql

2. **`desktop/src-tauri/src/db/connection.rs`**

   - Simplify DbState
   - Remove sync functions
   - Clean up initialize()

3. **`desktop/src-tauri/src/commands/sync.rs`**

   - Delete file

4. **`desktop/src-tauri/src/commands/mod.rs`**

   - Remove sync module

5. **`desktop/src-tauri/src/lib.rs`**

   - Remove sync command registrations

6. **`desktop/src-tauri/src/handlers/sync.rs`**

   - Delete file

7. **`desktop/src-tauri/src/handlers/mod.rs`**

   - Remove sync module

8. **`desktop/src-tauri/src/api/openapi.rs`**

   - Remove sync endpoints

### Frontend Files

**No frontend changes** - Frontend sync UI will remain but API endpoints will be removed, so sync functionality will not work.

### Documentation Files

1. **`FEATURES.md`**

   - Remove sync section

2. **`remote-replica/README.md`**

   - Add deprecation note

## Testing Checklist

- [ ] Database initializes in local-only mode
- [ ] All CRUD operations work (entries, tasks, goals, etc.)
- [ ] No sync-related errors in logs
- [ ] No sync API endpoints in OpenAPI spec
- [ ] Vector search still works (LibSQL feature, not sync)
- [ ] All repositories function correctly
- [ ] Application starts without sync warnings
- [ ] Frontend sync UI may show errors (expected - endpoints removed)

## Breaking Changes

1. **No sync capability** - Multi-device sync removed
2. **API changes** - Sync endpoints removed from API
3. **Frontend sync UI** - Will remain but will not function (endpoints removed)

## What Stays

- LibSQL dependency (local-only)
- Vector functions (`vector32`, `vector_distance_cos`, etc.) - these are LibSQL features, not sync features
- FTS5 search
- All existing database functionality
- Local database file

## Estimated Effort

- **Phase 1**: 1-2 hours (simplify connection.rs)
- **Phase 2**: 30 minutes (remove sync commands)
- **Phase 3**: 30 minutes (remove sync handlers)
- **Phase 4**: SKIPPED (frontend unchanged)
- **Phase 5**: 30 minutes (cleanup and docs)
- **Phase 6**: 30 minutes (remove comments)
- **Phase 7**: 30 minutes (error handling review)

**Total: 3.5-4.5 hours**