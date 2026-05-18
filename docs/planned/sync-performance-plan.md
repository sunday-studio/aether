# Sync Performance Plan

Sync latency is noticeable enough to treat as a product bug, not a later optimization pass. The main risk is that sync work currently appears to block normal app work while waiting on remote calls and serialized storage paths.

## Goals

- Make empty and small syncs feel instant in normal use.
- Keep the UI responsive while sync is pulling, pushing, or uploading media.
- Identify the real latency source with timing logs before deeper rewrites.
- Reduce request, database, and media overhead without changing sync semantics.

## Suspected Problems

### Global local database lock spans network work

The desktop backend serializes database access with a single `AsyncMutex`. `SyncEngine::sync_inner` and `push_pending` acquire that guard before remote pull, push, and media work. While that guard is held, normal frontend calls that need the database can queue behind network latency.

This is the highest priority suspect because it can make the app feel slow even when the sync server is only moderately slow.

### Sync HTTP clients are recreated per request

The sync helper constructs a fresh `reqwest::Client` for each pull, push, media upload, media download, and registration call. That prevents connection reuse and adds repeated setup cost.

### Push builds envelopes with N+1 database queries

The push path reads the outbox, then fetches the source row for each queued entity one at a time. It also deletes processed outbox rows one at a time after the server accepts the push.

### Media sync is sequential

When media sync policy is `auto`, media blobs are read, encrypted, and uploaded sequentially. One large file or slow upload can delay the whole metadata push.

### Sync server storage is serialized

The Axum sync server uses one `rusqlite::Connection` behind a `Mutex`. Authentication, push recording, pull queries, device updates, and blob metadata writes all contend on the same lock.

### Pull does extra server storage work

The server pull handler fetches rows, then checks whether more rows exist, then updates device sync metadata. This adds extra database calls per pull page.

## Phase 1: Add Timing Logs

Add elapsed-time logging around the sync pipeline before changing behavior.

Desktop timings:

- `sync_total`
- `pull_http`
- `pull_decode`
- `apply_changes`
- `push_outbox_build`
- `push_encrypt`
- `push_http`
- `push_delete_outbox`
- `media_upload_total`
- `media_download_total`

Server timings:

- `auth`
- `register_device`
- `record_push`
- `pull_query`
- `pull_encode`
- `has_more`
- `last_sync_update`
- `blob_put`
- `blob_get`

Expected outcome: a local log clearly shows whether the user-visible delay is lock wait, HTTP setup, SQLite work, JSON/base64/encryption, or media.

## Phase 2: Narrow Local Database Lock Scope

Do not hold `with_db_access` across network calls.

Proposed shape:

- Acquire the database lock briefly to read sync configuration, credentials, cursor, and outbox data.
- Release the lock before remote `/pull`, `/push`, and media transfer calls.
- Reacquire the lock only to apply pulled changes, update cursors, and delete accepted outbox rows.

This should make UI database calls responsive while sync waits on the server.

## Phase 3: Reuse A Shared HTTP Client

Create one shared sync `reqwest::Client` and pass it into register, pull, push, and media helpers.

Options:

- Store the client on `SyncEngine`.
- Use a `OnceLock<reqwest::Client>` for sync calls.

Keep the existing connect and request timeout behavior.

## Phase 4: Batch Push Work

Replace per-row push work with batched operations.

Proposed changes:

- Group outbox rows by entity.
- Fetch source rows per entity with `WHERE _sync_id IN (...)`.
- Build envelopes from the grouped result sets.
- Delete processed outbox rows in a transaction or with grouped deletes.

This should make large local edit batches scale with entity groups instead of individual changed rows.

## Phase 5: Improve Server Storage Concurrency

Reduce contention from the sync server's single `Mutex<Connection>`.

Options:

- Move blocking `rusqlite` work into `tokio::task::spawn_blocking`.
- Use a small SQLite connection pool.
- Keep write transactions short and allow read operations to proceed concurrently where SQLite WAL permits it.

This should help when multiple devices sync or when WebSocket pings/device updates overlap with push and pull.

## Phase 6: Reduce Pull Round Trips

Fetch `limit + 1` rows in `/pull`, return at most `limit`, and derive `has_more` from the extra row. This removes the separate `has_more_after` query.

Keep device `last_sync` updates cheap and avoid making them part of the critical path if they are only observability metadata.

## Phase 7: Make Media Sync Less Blocking

For `auto` media sync:

- Cap concurrent uploads and downloads with a small limit, such as 2 to 4.
- Keep metadata sync from being blocked behind large media files when possible.
- Consider checking whether the server already has a blob before uploading large encrypted media.

## Verification Plan

Create a repeatable local benchmark covering:

- Empty sync.
- 10 metadata changes.
- 500 metadata changes.
- 500 metadata changes plus media.
- Concurrent frontend query while sync is running.
- Two devices syncing against the same local server.

Target behavior:

- Empty sync is effectively instant locally.
- Small sync completes in under one second locally.
- UI queries do not stall while sync waits on remote HTTP or media transfers.
- Large sync scales predictably with row count and media size.

## Recommended Execution Order

1. Add timing logs.
2. Narrow the local database lock scope.
3. Reuse one sync HTTP client.
4. Batch push outbox reads and deletes.
5. Improve server storage concurrency.
6. Reduce pull storage round trips.
7. Add bounded media concurrency.
8. Run the benchmark and tune from measured results.
