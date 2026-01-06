# LibSQL Embedded Replica Mode

## Overview

Embedded replica mode **completely eliminates** `STREAM_EXPIRED` errors by creating a local SQLite database that automatically syncs with the remote libSQL server. This provides the best of both worlds:

- **Fast Reads**: All reads happen from local file (no network latency or stream timeouts)
- **Automatic Sync**: Writes are pushed to remote and synced back every 5 seconds
- **Offline Capability**: App can read stale data even when remote is unavailable
- **No Connection Pool Issues**: File-based access means no HTTP streams to expire

## How It Works

```
┌─────────────┐         sync every 5s         ┌──────────────┐
│   Local     │ ◄─────────────────────────── │   Remote     │
│  Replica    │                               │  libSQL      │
│  (SQLite)   │ ──────────────────────────► │   Server     │
└─────────────┘         writes               └──────────────┘
      ▲
      │ reads (instant, no network)
      │
 ┌────┴─────┐
 │   App    │
 └──────────┘
```

## Environment Variables

### Embedded Replica Mode (Recommended)

```bash
LIBSQL_URL=http://127.0.0.1:8080
LIBSQL_AUTH_TOKEN=your-token-here     # optional
LIBSQL_USE_REPLICA=true               # enables embedded replica
```

### Direct HTTP Mode (Fallback)

```bash
LIBSQL_URL=http://127.0.0.1:8080
LIBSQL_AUTH_TOKEN=your-token-here     # optional
# LIBSQL_USE_REPLICA not set or false
```

**Note**: Direct HTTP mode uses aggressive connection pooling (`MaxIdleConns=0`) to prevent stream expiration, but may still experience occasional errors under high load or network issues.

## File Structure

When using embedded replica mode, a local database is created:

```
backend/
├── libsql-replica/
│   └── local.db              # Local replica database
│   └── local.db-shm          # SQLite shared memory
│   └── local.db-wal          # SQLite write-ahead log
├── internal/
│   └── db/
│       └── db.go
└── main.go
```

**Important**: Add `libsql-replica/` to your `.gitignore` file.

## Connection URL Format

The embedded replica uses a special connection URL format:

```
file:./libsql-replica/local.db?sync=http://remote-url&authToken=token&syncInterval=5
```

Parameters:
- `file:path` - Local database file path
- `sync` - Remote libSQL server URL
- `authToken` - Authentication token (optional)
- `syncInterval` - Sync frequency in seconds (default: 5)

## Connection Pool Settings

### Replica Mode
```go
MaxIdleConns:     10           // Keep connections ready
MaxOpenConns:     50           // Allow concurrent operations
ConnMaxLifetime:  10 minutes   // Long-lived connections are fine
ConnMaxIdleTime:  5 minutes    // Keep idle connections
```

### Direct HTTP Mode
```go
MaxIdleConns:     0            // Close immediately
MaxOpenConns:     5            // Minimal concurrency
ConnMaxLifetime:  30 seconds   // Aggressive recycling
ConnMaxIdleTime:  0            // No idle connections
```

## Troubleshooting

### Stream Expired Errors (Direct HTTP Mode)

If you see `STREAM_EXPIRED` errors:

1. **Switch to embedded replica mode** (recommended):
   ```bash
   export LIBSQL_USE_REPLICA=true
   ```

2. **Or reduce connection reuse** (if replica mode is not possible):
   - Lower `MaxOpenConns` to 1-3
   - Ensure `MaxIdleConns=0`

### Sync Issues (Replica Mode)

If replica is not syncing:

1. Check network connectivity to remote server
2. Verify `LIBSQL_URL` and `LIBSQL_AUTH_TOKEN` are correct
3. Check logs for sync errors
4. Ensure remote server is running and accessible

### Stale Data

The replica syncs every 5 seconds by default. If you need fresher data:

1. Reduce `syncInterval` (minimum: 1 second)
2. Or use direct HTTP mode for critical real-time operations

## Performance Comparison

| Mode | Read Latency | Write Latency | Stream Errors | Offline |
|------|-------------|---------------|---------------|---------|
| **Embedded Replica** | ~0.1ms (local) | ~10-50ms (remote) | ❌ Never | ✅ Reads only |
| **Direct HTTP** | ~10-50ms (network) | ~10-50ms (network) | ⚠️ Possible | ❌ No |

## Migration from Direct HTTP to Replica

1. Add environment variable:
   ```bash
   export LIBSQL_USE_REPLICA=true
   ```

2. Restart the application

3. Initial sync will populate local replica (~1-10 seconds depending on data size)

4. Application is ready when you see:
   ```
   Embedded replica configured local_path=./libsql-replica/local.db sync_interval=5s
   Connection pool configured for embedded replica mode
   ```

## Best Practices

1. **Always use replica mode in production** to avoid stream expiration errors
2. **Add `libsql-replica/` to `.gitignore`** to avoid committing local database (already done)
3. **Monitor sync lag** if data freshness is critical (5 second default)
4. **Backup remote server** - replica is ephemeral and rebuilt on startup
5. **Use local-only mode for development** - just omit `LIBSQL_URL` to use local SQLite

## References

- [LibSQL Embedded Replicas Documentation](https://github.com/tursodatabase/libsql)
- [go-libsql Driver](https://github.com/tursodatabase/go-libsql)

