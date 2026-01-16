---
name: Backend Rust Migration Plan
overview: Migrate the Go backend to Rust within the desktop directory, using libsql crate for database access and maintaining OpenAPI spec generation. The migration will be done incrementally across multiple milestones to allow gradual transition.
todos:
  - id: milestone-1-foundation
    content: "Milestone 1: Set up Rust backend crate structure, database connection with libsql, error handling, logging, and health check endpoint"
    status: completed
  - id: milestone-2-models
    content: "Milestone 2: Port all database models, implement migration system, and create repository/query helpers"
    status: completed
    dependencies:
      - milestone-1-foundation
  - id: milestone-3-openapi-basic
    content: "Milestone 3: Set up OpenAPI/utoipa, port Tag and Entry handlers with OpenAPI annotations"
    status: completed
    dependencies:
      - milestone-2-models
  - id: milestone-4-tasks
    content: "Milestone 4: Port all task-related handlers including subtasks and task relationships (tags, goals)"
    status: completed
    dependencies:
      - milestone-3-openapi-basic
  - id: milestone-5-goals
    content: "Milestone 5: Port goal handlers, goal instances, and recurrence logic"
    status: in_progress
    dependencies:
      - milestone-4-tasks
  - id: milestone-6-remaining
    content: "Milestone 6: Port trash and sync handlers, complete all remaining endpoints"
    status: pending
    dependencies:
      - milestone-5-goals
  - id: milestone-7-integration
    content: "Milestone 7: Integration testing, update desktop app, verify OpenAPI spec matches, create migration guide"
    status: pending
    dependencies:
      - milestone-6-remaining
---

# Backend Rust Migration Plan

## Overview

This plan migrates the Go backend (`backend/`) to Rust within the `desktop/src-tauri/` directory structure. The Rust backend will use the `libsql` crate for database access, maintain OpenAPI spec generation, and be organized into milestones for gradual migration.

**Key Principles:**

- **Rust best practices**: Idiomatic Rust code, not a one-to-one translation from Go
- **Performance**: Zero-cost abstractions, prepared statements, efficient queries
- **Maintainability**: Clear module structure, repository pattern, type safety
- **Functionality over form**: Match Go backend functionality, not implementation details
- **Fast and maintainable**: Code written for long-term maintenance in Rust

## Architecture

The Rust backend will be structured as a library crate within the Tauri workspace, following Rust best practices:

- **Repository pattern** for database access (separation of concerns, testability)
- **Type-safe models** with serde for serialization
- **Zero-cost abstractions** where possible
- **Async/await** throughout for performance
- **Prepared statements** for query performance
- **Clear module boundaries** for maintainability
```
desktop/src-tauri/
├── Cargo.toml (workspace root)
├── src/ (existing Tauri app)
└── backend/ (new Rust backend crate)
    ├── Cargo.toml
    ├── src/
    │   ├── main.rs (standalone server)
    │   ├── lib.rs (library entry)
    │   ├── api/
    │   │   ├── mod.rs
    │   │   ├── routes.rs (route registration)
    │   │   └── extractors.rs (custom extractors if needed)
    │   ├── db/
    │   │   ├── mod.rs
    │   │   ├── connection.rs (libsql setup, sync)
    │   │   ├── models.rs (data models with serde)
    │   │   ├── repositories/ (repository pattern)
    │   │   │   ├── mod.rs
    │   │   │   ├── entry.rs
    │   │   │   ├── tag.rs
    │   │   │   ├── task.rs
    │   │   │   └── goal.rs
    │   │   └── migrations.rs (migration runner)
    │   ├── handlers/
    │   │   ├── mod.rs
    │   │   ├── entry.rs (thin handlers, delegate to repositories)
    │   │   ├── tag.rs
    │   │   ├── task.rs
    │   │   ├── goal.rs
    │   │   ├── trash.rs
    │   │   └── sync.rs
    │   ├── utils/
    │   │   ├── mod.rs
    │   │   ├── uuid.rs
    │   │   ├── timezone.rs
    │   │   └── goal_period.rs (goal period calculations)
    │   └── error.rs (centralized error types)
    └── migrations/ (SQL migration files)
        ├── 001_initial_schema.sql
        └── ...
```


## Technology Stack

- **Web Framework**: `axum` (async, modern, zero-cost abstractions, tokio-native)
- **Database**: `libsql` crate (with embedded replica support, direct connection usage)
- **Query Pattern**: Repository pattern with prepared statements for performance
- **OpenAPI**: `utoipa` + `utoipa-swagger-ui` (compile-time OpenAPI generation)
- **Serialization**: `serde` + `serde_json` (zero-copy where possible)
- **UUID**: `uuid` crate with v4 for IDs
- **Logging**: `tracing` + `tracing-subscriber` (structured, async-friendly)
- **Migrations**: Custom migration system with SQL files (simple, maintainable)
- **Environment**: `dotenvy` (for .env support)
- **Error Handling**: `thiserror` + `anyhow` (type-safe, zero-cost)
- **Time**: `chrono` + `chrono-tz` (timezone-aware, DST-safe)

## Milestone Breakdown

### Milestone 1: Foundation & Infrastructure

**Goal**: Set up Rust backend structure, database connection, and basic infrastructure

**Tasks**:

1. Create `backend/` crate in `desktop/src-tauri/`
2. Set up `Cargo.toml` with dependencies (axum, libsql, sqlx, utoipa, etc.)
3. Implement database connection module using libsql crate:

   - Support local-only mode (no LIBSQL_URL)
   - Support embedded replica mode (with LIBSQL_URL)
   - Background sync functionality
   - Connection pooling

4. Create basic error handling types
5. Set up logging with tracing
6. Create utility modules (UUID generation, timezone handling)
7. Implement basic health check endpoint (`/v1/ping`)

**Deliverables**:

- Working Rust backend that can start and connect to libSQL
- Health check endpoint responding
- Environment variable support (LIBSQL_URL, LIBSQL_AUTH_TOKEN, etc.)

### Milestone 2: Models & Migrations

**Goal**: Port database models and migration system using Rust best practices

**Tasks**:

1. Port all models from [backend/internal/db/models.go](backend/internal/db/models.go):

   - Entry, Tag, Task, SubTask, Goal, GoalInstance, Settings, SchemaMigration
   - Use `#[derive(Serialize, Deserialize)]` for JSON
   - Use `Option<T>` for nullable fields (not pointers)
   - Use `chrono::DateTime<Utc>` for timestamps
   - Use proper Rust naming conventions (snake_case)

2. Create SQL migration files for schema:

   - Initial schema migration (all tables)
   - Port existing Go migrations to SQL files
   - Use proper SQLite types (TEXT, INTEGER, REAL, BLOB)
   - Include indexes for performance

3. Implement migration runner:

   - Read migration files from `migrations/` directory
   - Track applied migrations in `schema_migrations` table
   - Run pending migrations on startup (fail fast on error)
   - Support both transactional and non-transactional migrations
   - Idempotent migrations (safe to run multiple times)

4. Create repository pattern:

   - `db/repositories/` module with per-entity repositories
   - Each repository takes `&Database` and returns `Result<T>`
   - Use prepared statements for all queries
   - Type-safe query methods (no string concatenation)
   - Clear error handling

**Deliverables**:

- All models ported to Rust with proper types
- Migration system working with SQL files
- Repository pattern established
- Database schema matches Go backend (functionally equivalent)

### Milestone 3: OpenAPI Setup & Basic Handlers

**Goal**: Set up OpenAPI documentation and port basic CRUD handlers

**Tasks**:

1. Set up `utoipa` for OpenAPI spec generation:

   - Define API paths with `#[utoipa::path(...)]` attributes
   - Define schemas with `#[derive(ToSchema)]` on models
   - Configure Swagger UI endpoint at `/swagger/*`
   - Generate OpenAPI JSON at `/api-doc/openapi.json`

2. Create Tag repository:

   - `find_all()` - Get all tags
   - `create()` - Create tag
   - `bulk_create()` - Bulk create tags (use transaction)

3. Create Entry repository:

   - `find_all()` - Get entries with optional filters
   - `find_by_id()` - Get entry by ID
   - `create()` - Create entry
   - `bulk_create()` - Bulk create entries (use transaction)
   - `update()` - Update entry
   - `delete()` - Soft delete entry

4. Port Tag handlers (thin layer, delegate to repository):

   - `GET /v1/tags` - Get all tags
   - `POST /v1/tags` - Create tag
   - `POST /v1/tags/bulk-create` - Bulk create tags

5. Port Entry handlers (thin layer, delegate to repository):

   - `GET /v1/entry` - Get entries
   - `GET /v1/entry/:id` - Get entry by ID
   - `POST /v1/entry` - Create entry
   - `POST /v1/entry/bulk-create` - Bulk create entries
   - `PUT /v1/entry/:id` - Update entry
   - `DELETE /v1/entry/:id` - Delete entry

6. Implement request/response types:

   - Use `#[derive(ToSchema)]` for OpenAPI
   - Use `#[derive(Deserialize)]` for request bodies
   - Use `#[derive(Serialize)]` for responses
   - Proper error responses with status codes

7. Set up CORS middleware (already done in Milestone 1)

**Deliverables**:

- OpenAPI spec generation working with utoipa
- Swagger UI accessible at `/swagger/*`
- Tag and Entry repositories implemented
- Tag and Entry handlers functional
- OpenAPI spec matches Go backend (functionally equivalent)

### Milestone 4: Task Management Handlers

**Goal**: Port all task-related handlers

**Tasks**:

1. Port Task handlers:

   - `POST /v1/tasks` - Create task
   - `GET /v1/tasks/inbox` - Get inbox tasks
   - `GET /v1/tasks/overdue` - Get overdue tasks
   - `GET /v1/tasks/:id` - Get task by ID
   - `PUT /v1/tasks/:id` - Update task
   - `DELETE /v1/tasks/:id` - Delete task

2. Port Task-Tag relationship handlers:

   - `POST /v1/tasks/:id/tags` - Add tags to task
   - `DELETE /v1/tasks/:id/tags` - Remove tags from task

3. Port Task-Goal relationship handlers:

   - `POST /v1/tasks/:id/goal` - Add goal to task
   - `DELETE /v1/tasks/:id/goal` - Remove goal from task

4. Port SubTask handlers:

   - `GET /v1/tasks/:taskId/subtasks` - Get subtasks
   - `POST /v1/tasks/:taskId/subtasks` - Create subtask
   - `PUT /v1/tasks/:taskId/subtasks/:subtaskId` - Update subtask
   - `DELETE /v1/tasks/:taskId/subtasks/:subtaskId` - Delete subtask
   - `POST /v1/tasks/:taskId/subtasks/reorder` - Reorder subtasks

5. Port Entry-Tag relationship handlers:

   - `POST /v1/entry/:id/tags` - Add tags to entry
   - `DELETE /v1/entry/:id/tags` - Remove tags from entry

**Deliverables**:

- All task-related endpoints functional
- Subtask management working
- Task-tag and task-goal relationships working

### Milestone 5: Goal Management Handlers

**Goal**: Port goal-related handlers and business logic

**Tasks**:

1. Port Goal handlers:

   - `GET /v1/goals` - Get goals
   - `GET /v1/goals/:id` - Get goal by ID
   - `POST /v1/goals` - Create goal
   - `PUT /v1/goals/:id` - Update goal
   - `DELETE /v1/goals/:id` - Delete goal

2. Port Goal Instance handlers:

   - `GET /v1/goals/:goalId/instances` - Get goal instances
   - `GET /v1/goals/:goalId/instances/current` - Get current goal instance

3. Port goal instance creation logic:

   - Move business logic to repository or utility module
   - Use Rust enums for recurrence types (type-safe)
   - Implement instance creation with proper error handling

4. Port goal period calculation utilities:

   - Create `utils/goal_period.rs` module
   - Use `chrono-tz` for timezone-aware calculations
   - Implement recurrence logic (daily, weekly, bi-weekly, monthly, yearly, custom)
   - DST-safe calculations (use chrono's timezone support)

5. Implement recurrence logic:

   - Use Rust enums instead of strings (compile-time safety)
   - Pattern matching for different recurrence types
   - Efficient date calculations (no unnecessary allocations)

**Deliverables**:

- All goal endpoints functional
- Goal instance management working
- Recurrence logic implemented

### Milestone 6: Trash & Sync Handlers

**Goal**: Port remaining handlers

**Tasks**:

1. Port Trash handlers:

   - `GET /v1/trash/tasks` - Get trashed tasks
   - `POST /v1/trash/tasks/:id/restore` - Restore task

2. Port Sync handler:

   - `POST /v1/sync` - Manual sync trigger

3. Ensure all handlers match Go backend behavior

**Deliverables**:

- Trash functionality working
- Sync endpoint functional
- All endpoints ported

### Milestone 7: Integration & Testing

**Goal**: Integrate with desktop app and ensure compatibility

**Tasks**:

1. Update desktop app to point to Rust backend (or run both in parallel)
2. Test all endpoints match Go backend responses
3. Verify OpenAPI spec matches Go backend spec
4. Test libSQL sync functionality
5. Performance testing and optimization
6. Update documentation
7. Create migration guide for switching from Go to Rust backend

**Deliverables**:

- Rust backend fully functional and tested
- Desktop app works with Rust backend
- Documentation updated
- Migration guide created

## Key Implementation Details

### Database Connection (libsql)

**Rust Best Practices:**

- Use `libsql::Database` directly (no ORM overhead)
- Connection per request pattern (libsql handles pooling internally)
- Prepared statements for all queries (performance + safety)
- Arc<Database> for shared state (zero-cost cloning)
- Background sync in separate tokio task (non-blocking)

**Implementation:**

1. **Local-only mode**: When `LIBSQL_URL` is not set, use `Builder::new_local()`
2. **Embedded replica mode**: When `LIBSQL_URL` is set, use `Builder::new_local_replica()` with sync
3. **Background sync**: Spawn tokio task with interval (non-blocking, handles errors gracefully)
4. **Connection management**: libsql handles connection pooling internally
5. **SQLite optimizations**: Apply PRAGMA settings on initialization

**Key differences from Go:**

- No GORM - use direct SQL with prepared statements
- Repository pattern instead of direct handler queries
- Type-safe models with serde (not struct tags)
- Async throughout (no blocking operations)

### OpenAPI Generation

Use `utoipa` for OpenAPI spec generation:

```rust
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::entry::get_entries,
        handlers::entry::create_entry,
        // ... all endpoints
    ),
    components(schemas(Entry, Tag, Task, /* ... */)),
    tags(
        (name = "Entries", description = "Entry management endpoints"),
        // ...
    ),
)]
struct ApiDoc;

// In main.rs
app = app.merge(
    SwaggerUi::new("/swagger/*")
        .url("/api-doc/openapi.json", ApiDoc::openapi())
);
```

### Migration System

**Rust Best Practices:**

- SQL files in `migrations/` directory (versioned, timestamped)
- Track applied migrations in `schema_migrations` table
- Run migrations on startup (fail fast if migrations fail)
- Support both transactional and non-transactional migrations
- Use libsql's transaction support where possible

**Implementation:**

```rust
// migrations/001_initial_schema.sql
CREATE TABLE IF NOT EXISTS entries (
    id TEXT PRIMARY KEY,
    document TEXT NOT NULL,
    created_at TEXT NOT NULL,
    -- ...
);

// db/migrations.rs
pub async fn run_migrations(db: &Database) -> Result<()> {
    // Ensure schema_migrations table exists
    // Read migration files from migrations/ directory
    // Check which migrations are already applied
    // Run pending migrations in order
    // Record in schema_migrations table
    // Support NoTransaction flag for PRAGMA statements
}
```

**Key differences from Go:**

- File-based migrations (not code-based)
- Simpler, more maintainable
- Version tracking in database
- Fail-fast on migration errors

## Environment Variables

Maintain compatibility with Go backend:

- `LIBSQL_URL` - Remote libSQL URL (optional)
- `LIBSQL_AUTH_TOKEN` - Authentication token (optional)
- `LIBSQL_SYNC_INTERVAL` - Sync interval in seconds (default: 10)
- `PORT` - Server port (default: 9119)

## Rust Best Practices & Performance

### Code Quality

- **Zero-cost abstractions**: Use Rust's type system, not runtime checks
- **Ownership**: Leverage Rust's ownership for memory safety without GC overhead
- **Error handling**: Use `Result<T, E>` everywhere, no panics in production code
- **Async/await**: All I/O operations are async (non-blocking)
- **Type safety**: Leverage Rust's type system to prevent bugs at compile time

### Performance Optimizations

- **Prepared statements**: All queries use prepared statements (compiled once, reused)
- **Connection reuse**: libsql handles connection pooling efficiently
- **Zero-copy deserialization**: Use serde's zero-copy features where possible
- **Arc for shared state**: Zero-cost cloning of Database handle
- **Efficient queries**: Write SQL queries that leverage indexes
- **Batch operations**: Use transactions for bulk operations

### Maintainability

- **Repository pattern**: Clear separation between data access and business logic
- **Type-safe models**: Models with serde derive, no runtime reflection
- **Module structure**: Clear module boundaries, easy to navigate
- **Error types**: Centralized error handling with thiserror
- **Documentation**: Use rustdoc for all public APIs

### Differences from Go Implementation

- **No ORM**: Direct SQL with prepared statements (more control, better performance)
- **Repository pattern**: Instead of handlers directly querying database
- **Type safety**: Compile-time guarantees instead of runtime checks
- **Async-first**: Everything is async, no blocking operations
- **Ownership**: No need for manual memory management or GC pauses

## Testing Strategy

1. **Unit tests**: Test repositories, utilities, and business logic in isolation
2. **Integration tests**: Test database operations with in-memory/test database
3. **API tests**: Test endpoints with test client (verify JSON responses match)
4. **Migration tests**: Verify migrations work correctly and are idempotent
5. **Performance tests**: Benchmark critical paths (queries, serialization)

## Rollout Strategy

1. Run Rust backend on different port initially (e.g., 9120)
2. Test in parallel with Go backend
3. Gradually switch desktop app to use Rust backend
4. Once stable, deprecate Go backend

## Files to Reference

- [backend/main.go](backend/main.go) - Main entry point
- [backend/internal/api/api.go](backend/internal/api/api.go) - Route registration
- [backend/internal/db/db.go](backend/internal/db/db.go) - Database initialization
- [backend/internal/db/models.go](backend/internal/db/models.go) - Data models
- [backend/internal/handlers/](backend/internal/handlers/) - All handler implementations
- [backend/docs/swagger.yaml](backend/docs/swagger.yaml) - OpenAPI spec reference