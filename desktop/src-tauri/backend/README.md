# Aether Backend (Rust)

This is the Rust implementation of the Aether backend, migrated from Go. It provides a high-performance, type-safe API server for the Aether desktop application.

## Features

- **Full API Compatibility**: Maintains 100% compatibility with the Go backend
- **Offline-First**: Local database with optional sync to Turso
- **Type Safety**: Rust's compile-time guarantees prevent many runtime errors
- **Performance**: Zero-cost abstractions and efficient async runtime
- **OpenAPI**: Auto-generated API documentation via `utoipa`

## Quick Start

### Running the Server

```bash
cd desktop/src-tauri/backend
cargo run --bin aether-backend
```

The server will start on `http://localhost:9119` by default.

### Environment Variables

- `PORT`: Server port (default: 9119)
- `LIBSQL_SYNC_INTERVAL`: Sync interval in seconds (default: 10)
- `RUST_LOG`: Logging level (default: `aether_backend=info`)

### API Documentation

Once the server is running, visit:
- **Swagger UI**: http://localhost:9119/swagger/
- **OpenAPI JSON**: http://localhost:9119/api-doc/openapi.json

## Project Structure

```
backend/
├── src/
│   ├── main.rs              # Binary entry point
│   ├── lib.rs               # Library entry point
│   ├── api/                 # API routes and OpenAPI
│   ├── db/                  # Database models, repositories, migrations
│   ├── handlers/            # HTTP request handlers
│   ├── utils/               # Utility functions
│   └── error.rs             # Error types
├── migrations/              # SQL migration files
└── tests/                   # Integration tests
```

## Development

### Building

```bash
cargo build
```

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests (requires server running)
TEST_API_URL=http://localhost:9119 cargo test --test integration_test -- --ignored
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Database

### Local-Only Mode (Default)

The backend starts in local-only mode, storing data in `./libsql-replica/local.db`. This provides:

- Fast local reads/writes
- No network dependency
- Full offline capability

### Enabling Sync

To enable sync with a remote Turso database:

```bash
curl -X POST http://localhost:9119/v1/sync/configure \
  -H "Content-Type: application/json" \
  -d '{
    "sync_url": "libsql://your-database.turso.io",
    "auth_token": "your-token"
  }'
```

Once configured, the backend will:
- Continue writing to local database first (offline writes)
- Automatically sync changes to remote
- Pull remote changes periodically

## Migration from Go Backend

See [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md) for detailed migration instructions.

## API Endpoints

All endpoints are prefixed with `/v1`:

- **Health**: `GET /v1/ping`
- **Tags**: `GET|POST /v1/tags`, `POST /v1/tags/bulk-create`
- **Entries**: `GET|POST /v1/entry`, `GET|PUT|DELETE /v1/entry/:id`, `POST|DELETE /v1/entry/:id/tags`
- **Tasks**: `POST /v1/tasks`, `GET /v1/tasks/inbox`, `GET /v1/tasks/overdue`, etc.
- **Goals**: `GET|POST /v1/goals`, `GET|PUT|DELETE /v1/goals/:id`, etc.
- **Trash**: `GET /v1/trash/tasks`, `POST /v1/trash/:id/restore`
- **Sync**: `POST /v1/sync`, `POST /v1/sync/configure`

See the Swagger UI for complete API documentation.

## Dependencies

- **axum**: Modern async web framework
- **libsql**: Database driver with embedded replica support
- **utoipa**: OpenAPI specification generation
- **tokio**: Async runtime
- **serde**: Serialization/deserialization
- **chrono**: Date/time handling

## License

Same as the main Aether project.
