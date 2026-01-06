# libSQL Migration Implementation

This document describes the libSQL migration implementation for the Aether backend.

## What Was Implemented

### 1. Database Driver Update
- Added `github.com/tursodatabase/go-libsql` dependency
- Updated `backend/internal/db/db.go` to support both libSQL and SQLite
- Uses `libsql.NewRemoteConnector()` for remote libSQL server connections
- Backend automatically uses libSQL if `LIBSQL_URL` environment variable is set
- Falls back to local SQLite if `LIBSQL_URL` is not set (for development)
- **Installation**: Run `go get github.com/tursodatabase/go-libsql && go mod tidy`

### 2. Last-Write-Wins Conflict Resolution
Implemented LWW conflict detection in all update handlers:

- **Entry Handler** (`backend/internal/handlers/entry/update_entry.go`)
  - Checks if client's `UpdatedAt` is older than server's
  - Returns 409 Conflict if client has stale data
  
- **Task Handler** (`backend/internal/handlers/task/update_task.go`)
  - Added `UpdatedAt` field to `UpdateTaskPayload`
  - Implements LWW conflict detection
  
- **Goal Handler** (`backend/internal/handlers/goal/update_goal.go`)
  - Added `UpdatedAt` field to `UpdateGoalPayload`
  - Implements LWW conflict detection

### 3. Infrastructure Setup
- Created `docker-compose.libsql.yml` for easy libSQL server setup
- Created `libsql-setup.md` with setup instructions
- Created `migrate-to-libsql.sh` script for data migration
- Updated `go.mod` with libSQL dependency

## How It Works

### Conflict Resolution Flow

1. Client sends update request with `UpdatedAt` timestamp from last known state
2. Server fetches current record from database
3. Server compares client's `UpdatedAt` with server's `UpdatedAt`:
   - If client's timestamp is **older**: Return 409 Conflict with current server data
   - If client's timestamp is **same or newer**: Apply update, GORM auto-updates `UpdatedAt`

### Client Response to Conflict

When client receives 409 Conflict:
```json
{
  "error": "conflict",
  "message": "Record was modified by another device",
  "current": { /* current server data */ }
}
```

Client should:
1. Refresh local data with the `current` object
2. Show user that data was modified by another device
3. Allow user to retry or merge changes

## Environment Variables

```bash
# Required for libSQL (optional - falls back to SQLite if not set)
LIBSQL_URL=http://localhost:8080
LIBSQL_AUTH_TOKEN=your-token-here  # Optional, if authentication enabled
```

## Setup Instructions

1. **Start libSQL server:**
   ```bash
   docker-compose -f docker-compose.libsql.yml up -d
   ```

2. **Set environment variables:**
   ```bash
   export LIBSQL_URL=http://localhost:8080
   ```

3. **Run migrations:**
   ```bash
   go run main.go
   # Migrations will run automatically
   ```

4. **Migrate existing data (if needed):**
   ```bash
   ./migrate-to-libsql.sh http://localhost:8080
   ```

## Testing

To test conflict resolution:

1. Start two instances of the app (or use two devices)
2. Fetch the same record on both
3. Modify the record on device 1
4. Try to modify the record on device 2 with the old `UpdatedAt`
5. Device 2 should receive 409 Conflict response

## Notes

- libSQL is SQLite-compatible, so existing GORM queries work without changes
- The implementation maintains backward compatibility with SQLite
- Desktop app continues using HTTP API (no changes needed)
- Future: Can add embedded libSQL replica to desktop for offline support

