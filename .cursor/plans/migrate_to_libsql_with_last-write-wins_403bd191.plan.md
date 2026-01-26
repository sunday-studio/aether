---
name: ""
overview: ""
todos: []
isProject: false
---

# Migrate to libSQL with Last-Write-Wins Conflict Resolution

## Status: DEFERRED

**This migration is planned for after the project reaches feature parity.** The plan is documented here for future implementation.

## Overview

Migrate the backend database from SQLite to libSQL (self-hosted) to enable built-in replication and multi-device sync. Implement last-write-wins conflict resolution using `UpdatedAt` timestamps.**Architecture Decision**: Using **Option A** - Keep current architecture where desktop app continues using HTTP API → Backend → libSQL Server. No desktop code changes needed initially.

## Architecture Changes

### Current Architecture

```javascript
Desktop App → HTTP API → Backend → SQLite file (aether.db)
                                    ↓
                              Single instance, no replication
```

### New Architecture (Phase 1: Backend Only)

```javascript
Desktop App → HTTP API → Backend → libSQL Server (self-hosted sqld)
                                    ↓
                              Better concurrency, ready for replication
```

### Future Architecture (Phase 2: Desktop Embedded Replica)

```javascript
Desktop App → Embedded libSQL → Syncs with → libSQL Server
     ↓                              ↓
Local DB (offline)            Backend → libSQL Server
```

## Implementation Steps

### Phase 1: Setup Self-Hosted libSQL Server

1. **Install `sqld` (libSQL server):**
   ```bash
                  # Option 1: Build from source
                  git clone https://github.com/tursodatabase/libsql.git
                  cd libsql
                  cargo build --release
                  
                  # Option 2: Use Docker (recommended)
                  docker pull ghcr.io/tursodatabase/libsql-server:latest
   ```

2. **Run libSQL server:**
   ```bash
                  # Docker approach
                  docker run -d \
                    --name libsql-server \
                    -p 8080:8080 \
                    -v $(pwd)/data:/data \
                    ghcr.io/tursodatabase/libsql-server:latest \
                    --grpc-listen-addr 0.0.0.0:5001 \
                    --http-listen-addr 0.0.0.0:8080 \
                    --data-dir /data
                  
                  # Or run sqld binary directly
                  ./sqld --grpc-listen-addr 0.0.0.0:5001 --http-listen-addr 0.0.0.0:8080
   ```

3. **Configure server:**

- HTTP endpoint: `http://your-server:8080`
- WebSocket endpoint: `ws://your-server:8080`
- GRPC endpoint: `0.0.0.0:5001` (for replication)
- Data directory: Persistent storage for database files

4. **Set up authentication** (optional but recommended):

- Generate JWT tokens for client authentication
- Configure server to require auth tokens

### Phase 2: Update Backend Dependencies

**File**: `backend/go.mod`Replace SQLite driver with libSQL driver:

- Remove: `gorm.io/driver/sqlite v1.6.0`
- Add: `github.com/tursodatabase/libsql-client-go` or use `github.com/tursodatabase/libsql/libsql-go`

**Note**: Check for GORM-compatible libSQL driver. If none exists, may need to use raw SQL or find alternative.

### Phase 3: Update Database Initialization

**File**: `backend/internal/db/db.go`Changes needed:

1. Replace SQLite driver import with libSQL driver
2. Update connection string to use Turso URL
3. Configure connection with auth token
4. Keep connection pooling settings (libSQL supports this)

**Example structure**:

```go
func Initialize() (*gorm.DB, error) {
    // Get from environment variables
    // For self-hosted: http://your-server:8080 or libsql://your-server:8080
    dbURL := os.Getenv("LIBSQL_URL") // http://localhost:8080 or libsql://...
    authToken := os.Getenv("LIBSQL_AUTH_TOKEN") // Optional, if auth enabled
    
    // Use libSQL driver instead of sqlite
    // Note: May need to use database/sql with libSQL client if GORM driver doesn't exist
    db, err := gorm.Open(libsql.Open(dbURL, authToken), &gorm.Config{
        Logger: gormLogger,
    })
    // ... rest of initialization
}
```

**Note**: Check if GORM libSQL driver exists. If not, you may need to:

- Use `database/sql` with `github.com/tursodatabase/libsql-client-go`
- Or create a custom GORM driver wrapper

### Phase 4: Implement Last-Write-Wins Conflict Resolution

#### 4.1 Update Models (Optional Enhancement)

**File**: `backend/internal/db/models.go`Models already have `UpdatedAt` field (managed by GORM), which is sufficient for LWW. Optionally add:

- `Version` field (integer, auto-increment on update) for additional conflict detection
- `DeviceID` field to track which device made changes (for logging/debugging)

#### 4.2 Update Entry Handler

**File**: `backend/internal/handlers/entry/update_entry.go`Implement LWW by checking `UpdatedAt` before update:

```go
func (e *EntryHandler) UpdateEntry(c *fiber.Ctx) error {
    var entry db.Entry
    if err := e.db.First(&entry, "id = ? AND is_deleted = ?", c.Params("id"), false).Error; err != nil {
        // ... error handling
    }

    var payload db.Entry
    if err := c.BodyParser(&payload); err != nil {
        return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
    }

    // Last-Write-Wins: Check if client's UpdatedAt is older than server's
    if !payload.UpdatedAt.IsZero() && payload.UpdatedAt.Before(entry.UpdatedAt) {
        // Client has stale data, return current server version
        return c.Status(409).JSON(fiber.Map{
            "error": "conflict",
            "message": "Record was modified by another device",
            "current": entry,
        })
    }

    // Update fields
    entry.Document = payload.Document
    entry.IsPinned = payload.IsPinned
    entry.IsArchived = payload.IsArchived
    entry.IsDeleted = payload.IsDeleted
    // UpdatedAt will be auto-updated by GORM

    if err := e.db.Save(&entry).Error; err != nil {
        return c.Status(500).JSON(fiber.Map{"error": err.Error()})
    }

    return c.JSON(entry)
}
```

#### 4.3 Update Task Handler

**File**: `backend/internal/handlers/task/update_task.go`Similar LWW implementation:

```go
func (h *TaskHandler) UpdateTask(c *fiber.Ctx) error {
    // ... existing code to get task
    
    // Last-Write-Wins: Check timestamp if provided in payload
    if payload.UpdatedAt != nil && !payload.UpdatedAt.IsZero() {
        if payload.UpdatedAt.Before(task.UpdatedAt) {
            return c.Status(409).JSON(fiber.Map{
                "error": "conflict",
                "message": "Task was modified by another device",
                "current": task,
            })
        }
    }
    
    // ... rest of update logic
}
```

#### 4.4 Update Other Handlers

Apply same pattern to:

- `backend/internal/handlers/goal/update_goal.go`
- Any other update handlers

### Phase 5: Environment Configuration

**File**: `backend/.env` or environment variablesAdd:

```javascript
# Self-hosted libSQL server
LIBSQL_URL=http://your-server:8080
# Or: libsql://your-server:8080
LIBSQL_AUTH_TOKEN=your-auth-token  # Optional if auth enabled
```

Update deployment scripts to include these variables.

### Phase 6: Data Migration

1. **Export existing SQLite data:**
   ```bash
                     sqlite3 sqlite/aether.db .dump > aether_backup.sql
   ```

2. **Import to libSQL:**

- Use Turso CLI or libSQL client to import SQL dump
- Or write migration script to transfer data

3. **Verify data integrity:**

- Check record counts
- Verify relationships (tags, many-to-many)

### Phase 7: Desktop App Integration Options

You have **three options** for how the desktop app interfaces with libSQL:

#### Option A: Keep Current Architecture (Simplest - Recommended for Phase 1)

**Desktop → HTTP API → Backend → libSQL Server**

- **Pros**: 
- No desktop code changes needed
- Backend handles all DB logic and validation
- Simpler to implement initially
- **Cons**: 
- Desktop always needs network connection
- No offline support
- Slower (API overhead)
- **Implementation**: No changes to desktop app, backend migration is sufficient

#### Option B: Desktop Embedded Replica (Best for Offline - Phase 2)

**Desktop → Embedded libSQL → Syncs with Server**

- **Pros**: 
- Full offline support
- Fast local reads/writes
- Automatic background sync
- Local-first architecture
- **Cons**: 
- More complex setup
- Need to handle sync conflicts in desktop
- Larger app bundle (includes libSQL)
- **Implementation**:

1. Install `@libsql/client` in desktop app:
     ```bash
                              cd desktop
                              npm install @libsql/client
     ```

2. Create libSQL client wrapper:
     ```typescript
                              // desktop/src/lib/libsql-client.ts
                              import { createClient } from '@libsql/client';
                              
                              const client = createClient({
                                url: 'file:./local-aether.db', // Local embedded database
                                syncUrl: 'http://your-server:8080', // Sync with server
                                authToken: process.env.LIBSQL_AUTH_TOKEN,
                              });
                              
                              // Sync in background
                              await client.sync();
     ```

3. Update React Query to use local replica:

    - Query local DB first (fast, works offline)
    - Sync in background periodically
    - Fallback to API if sync fails

#### Option C: Desktop Direct Connection (Not Recommended)

**Desktop → Direct libSQL Connection (bypasses backend)**

- **Pros**: Fastest, no API overhead
- **Cons**: 
- Bypasses backend business logic
- Security concerns (direct DB access)
- No API layer for validation/auth
- **Implementation**: Not recommended for your use case

#### Recommended Approach

**Phase 1 (After Project Parity)**: Use Option A

- Keep current HTTP API architecture
- Backend migration to libSQL provides better concurrency
- No desktop code changes needed
- Desktop continues: `Desktop → HTTP API → Backend → libSQL Server`

**Phase 2 (Future Enhancement)**: Migrate to Option B

- Add embedded libSQL replica to desktop
- Enable full offline support
- Requires additional implementation work
- Desktop becomes: `Desktop → Embedded libSQL → Syncs with Server`

## Conflict Resolution Strategy: Last-Write-Wins

### How It Works

1. **Client sends update** with `UpdatedAt` timestamp from last known state
2. **Server checks** if client's `UpdatedAt` is older than server's current `UpdatedAt`
3. **If conflict detected** (client has older data):

- Return 409 Conflict with current server data
- Client should refresh and retry

4. **If no conflict** (client has same or newer timestamp):

- Apply update
- GORM auto-updates `UpdatedAt` to current time

### Edge Cases to Handle

1. **Clock skew**: Devices may have slightly different times

- Solution: Use server time as source of truth, add small buffer (e.g., 5 seconds)

2. **Concurrent updates**: Two devices update simultaneously

- Solution: libSQL handles write ordering, LWW ensures latest wins

3. **Offline updates**: Device updates offline, then syncs

- Solution: libSQL replication handles this, LWW resolves conflicts

## Testing Checklist

- [ ] Backend connects to libSQL successfully
- [ ] Migrations run correctly
- [ ] CRUD operations work
- [ ] LWW conflict detection works (test with concurrent updates)
- [ ] Data migration completed successfully
- [ ] Performance is acceptable
- [ ] Error handling works correctly

## Rollback Plan

If issues arise:

1. Keep SQLite backup
2. Revert `go.mod` to SQLite driver
3. Update `db.go` to use SQLite
4. Restore from backup if needed

## Files to Modify

1. `backend/go.mod` - Update dependencies
2. `backend/internal/db/db.go` - Change driver and connection
3. `backend/internal/handlers/entry/update_entry.go` - Add LWW logic
4. `backend/internal/handlers/task/update_task.go` - Add LWW logic
5. `backend/internal/handlers/goal/update_goal.go` - Add LWW logic (if exists)
6. `backend/deploy.sh` - Add environment variables
7. `.env.example` - Document new variables

## Dependencies

**Backend:**

- libSQL Go client library (`github.com/tursodatabase/libsql-client-go`)
- Or GORM libSQL driver (if available)
- `sqld` server binary or Docker image

**Desktop (for future embedded replica):**

- `@libsql/client` npm package
- Tauri permissions for network access

**Infrastructure:**

- Server to host `sqld` (same as backend server or separate)
- Docker (optional, for easier deployment)

## Notes

- libSQL is SQLite-compatible, so most GORM queries should work
- May need to check for GORM driver compatibility
- Consider using `database/sql` with libSQL if GORM driver doesn't exist
- Self-hosted `sqld` server needs to be accessible from backend
- For desktop embedded replica, need to handle sync conflicts client-side
- Test thoroughly before deploying to production
- Consider firewall rules to restrict libSQL server access

## Desktop Integration Summary

**Phase 1 (After Project Parity)**: Keep current architecture

- Desktop continues using HTTP API → Backend → libSQL
- No desktop code changes needed
- Backend migration to libSQL provides better concurrency and prepares for future sync
- **Status**: Deferred until after project reaches feature parity

**Phase 2 (Future Enhancement)**: Add embedded replica

- Desktop gets local libSQL database
- Full offline support
- Automatic background sync
- **Status**: Will be implemented after Phase 1 is complete

## Implementation Timeline

1. **Current**: Focus on reaching project feature parity with existing SQLite setup