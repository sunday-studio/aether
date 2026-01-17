# Migration Guide: Go Backend to Rust Backend

This guide documents the migration from the Go backend (`backend/`) to the Rust backend (`desktop/src-tauri/backend/`).

## Overview

The Rust backend is a complete rewrite of the Go backend, maintaining API compatibility while leveraging Rust's performance, safety, and modern async capabilities. The backend uses:

- **libsql** for database operations (same as Go backend)
- **axum** for the HTTP server
- **utoipa** for OpenAPI specification generation
- **tokio** for async runtime

## Key Differences

### Architecture

1. **Repository Pattern**: The Rust backend uses a repository pattern for data access, similar to the Go backend but with Rust's type safety.

2. **Error Handling**: Uses `thiserror` for structured error handling with `Result<T>` types throughout.

3. **Database Connection**: 
   - Starts in local-only mode by default
   - Sync can be enabled dynamically via `/v1/sync/configure` endpoint
   - Uses `Arc<Mutex<>>` for thread-safe database swapping

4. **OpenAPI**: Automatically generated from code using `utoipa` annotations, ensuring API docs stay in sync with implementation.

## API Compatibility

The Rust backend maintains **full API compatibility** with the Go backend. All endpoints have the same:

- URL paths
- HTTP methods
- Request/response formats
- Status codes

### Endpoints

All endpoints from the Go backend are available:

- **Tags**: `/v1/tags`, `/v1/tags/bulk-create`
- **Entries**: `/v1/entry`, `/v1/entry/:id`, `/v1/entry/:id/tags`
- **Tasks**: `/v1/tasks`, `/v1/tasks/inbox`, `/v1/tasks/overdue`, `/v1/tasks/:id`, `/v1/tasks/:id/subtasks`, etc.
- **Goals**: `/v1/goals`, `/v1/goals/:id`, `/v1/goals/:goalId/instances`, etc.
- **Trash**: `/v1/trash/tasks`, `/v1/trash/:id/restore`
- **Sync**: `/v1/sync`, `/v1/sync/configure` (new)

### New Endpoint

- **`POST /v1/sync/configure`**: Configure sync with remote database dynamically
  ```json
  {
    "sync_url": "libsql://your-database.turso.io",
    "auth_token": "your-token"
  }
  ```

## Database Migration

### Schema

The database schema is identical. The Rust backend uses the same migration system:

1. SQL migration files in `migrations/` directory
2. Automatic migration tracking via `schema_migrations` table
3. Same table structure and constraints

### Data Migration

If you have existing data in the Go backend's database:

1. **Option 1: Use the same database file**
   - The Rust backend uses `./libsql-replica/local.db` by default
   - Copy your Go backend's database file to this location
   - The Rust backend will work with existing data

2. **Option 2: Export/Import**
   - Export data from Go backend (if needed)
   - Import into Rust backend database
   - Both use the same schema, so data should be compatible

### Sync Configuration

The Rust backend starts in **local-only mode**. To enable sync:

1. Start the Rust backend
2. Call `POST /v1/sync/configure` with your Turso database URL and auth token
3. Existing local data will be synced to remote automatically

## Running the Rust Backend

### Standalone Mode

```bash
cd desktop/src-tauri/backend
cargo run --bin aether-backend
```

The server will start on `http://localhost:9119` by default (configurable via `PORT` env var).

### Environment Variables

- `PORT`: Server port (default: 9119)
- `LIBSQL_SYNC_INTERVAL`: Sync interval in seconds (default: 10)
- `RUST_LOG`: Logging level (default: `aether_backend=info`)

### Integration with Tauri

The Rust backend is designed to run as a standalone server that the Tauri desktop app connects to via HTTP. The desktop app's API client (`src/lib/api-client.ts`) already uses HTTP, so no changes are needed.

## Testing

### Running Tests

```bash
cd desktop/src-tauri/backend
cargo test
```

### Integration Tests

Integration tests require the server to be running:

```bash
# Terminal 1: Start server
cargo run --bin aether-backend

# Terminal 2: Run tests
TEST_API_URL=http://localhost:9119 cargo test --test integration_test -- --ignored
```

## OpenAPI Specification

The OpenAPI spec is automatically generated and available at:

- **Swagger UI**: `http://localhost:9119/swagger/`
- **OpenAPI JSON**: `http://localhost:9119/api-doc/openapi.json`

To compare with the Go backend's spec:

```bash
# Rust backend
curl http://localhost:9119/api-doc/openapi.json > rust-openapi.json

# Compare (requires jq)
diff <(jq -S . rust-openapi.json) <(jq -S . backend/docs/swagger.json)
```

## Performance Considerations

### Advantages

- **Zero-cost abstractions**: Rust's compile-time optimizations
- **Memory safety**: No GC pauses
- **Concurrent operations**: Tokio's async runtime handles many concurrent requests efficiently
- **Type safety**: Compile-time guarantees prevent many runtime errors

### Benchmarks

The Rust backend should perform similarly or better than the Go backend for most operations, especially under high concurrency.

## Troubleshooting

### Database Connection Issues

If you encounter database connection errors:

1. Check that the `libsql-replica/` directory exists and is writable
2. Verify database file permissions
3. Check logs for detailed error messages

### Sync Issues

If sync is not working:

1. Verify the sync URL format: `libsql://your-database.turso.io`
2. Check that the auth token is correct
3. Ensure network connectivity to Turso
4. Check logs for sync errors

### Migration Issues

If migrations fail:

1. Check that the `migrations/` directory exists
2. Verify SQL file syntax
3. Check `schema_migrations` table for applied migrations
4. Review logs for specific error messages

## Rollback Plan

If you need to rollback to the Go backend:

1. Stop the Rust backend
2. Start the Go backend
3. Both use the same database file, so data should be preserved
4. Note: If sync was configured in Rust backend, you may need to reconfigure in Go backend

## Support

For issues or questions:

1. Check the logs: `RUST_LOG=debug cargo run --bin aether-backend`
2. Review the OpenAPI spec at `/swagger/`
3. Check the code documentation

## Next Steps

1. **Test the Rust backend** with your existing frontend
2. **Verify API compatibility** by comparing responses
3. **Configure sync** if you need remote database sync
4. **Monitor performance** and compare with Go backend
5. **Gradually migrate** or switch completely based on your needs
