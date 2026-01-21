# Quick Start Guide

## Running with LibSQL Embedded Replica (Recommended)

This setup eliminates `STREAM_EXPIRED` errors completely.

### 1. Start LibSQL Server

```bash
cd backend
docker-compose -f docker-compose.libsql.yml up -d
```

This starts a local libSQL server on `http://127.0.0.1:8080`

### 2. Set Environment Variables

```bash
export LIBSQL_URL=http://127.0.0.1:8080
export LIBSQL_USE_REPLICA=true
```

Or create a `.env` file:
```env
LIBSQL_URL=http://127.0.0.1:8080
LIBSQL_USE_REPLICA=true
```

### 3. Run the Backend

```bash
go run main.go
```

You should see:
```
Using libSQL embedded replica mode syncUrl=http://127.0.0.1:8080
Embedded replica connection string url=file:./libsql-replica/local.db?authToken=&sync=http://127.0.0.1:8080&syncInterval=5
Successfully connected to libSQL embedded replica
```

### 4. Verify It's Working

The `libsql-replica/` directory should be created with:
- `local.db` - Your local replica database
- `local.db-shm` - Shared memory file
- `local.db-wal` - Write-ahead log

All database operations now happen on the local file, with automatic sync to the remote server every 5 seconds.

## Alternative: Local SQLite (Development Only)

For simple local development without libSQL:

```bash
# Don't set LIBSQL_URL
unset LIBSQL_URL
go run main.go
```

This uses `./sqlite/aether.db` with no replication.

## Alternative: Direct HTTP Mode (Not Recommended)

If you can't use replica mode for some reason:

```bash
export LIBSQL_URL=http://127.0.0.1:8080
# Don't set LIBSQL_USE_REPLICA
go run main.go
```

⚠️ **Warning**: This mode uses aggressive connection pooling but may still experience occasional `STREAM_EXPIRED` errors under load.

## Troubleshooting

### "failed to create replica dir"
- Check file permissions in the backend directory
- Ensure you're running from the `backend/` directory

### "failed to open libSQL replica"
- Verify libSQL server is running: `docker ps`
- Check the URL is correct: `curl http://127.0.0.1:8080`
- Review logs: `docker logs libsql-server`

### Data not syncing
- Check `LIBSQL_URL` is accessible from the backend
- Verify no firewall blocking port 8080
- Look for sync errors in backend logs

### Still seeing STREAM_EXPIRED errors
- Confirm `LIBSQL_USE_REPLICA=true` is set
- Check logs to verify replica mode is active
- Restart the backend to ensure env vars are loaded

## Production Deployment

For production with Turso or remote libSQL:

```bash
export LIBSQL_URL=libsql://your-database.turso.io
export LIBSQL_AUTH_TOKEN=your-auth-token
export LIBSQL_USE_REPLICA=true
```

The embedded replica will sync with your remote database, providing fast local reads and automatic replication.

