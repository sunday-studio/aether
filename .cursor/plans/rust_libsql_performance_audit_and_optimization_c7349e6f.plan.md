---
name: Rust libSQL Performance Audit and Optimization
overview: Comprehensive performance audit of Rust libSQL codebase focusing on connection management, query optimization, transactions, pragmas, N+1 queries, and missing LIMITs. Addresses critical issues like missing WAL mode, N+1 query patterns, and query optimizations.
todos:
  - id: "1"
    content: Add WAL mode and optimize pragmas in connection.rs (journal_mode=WAL, cache_size=-64000, mmap_size=268435456)
    status: pending
  - id: "2"
    content: Fix N+1 queries in entry.rs::add_tags() - use single query with IN clause for tag verification
    status: pending
  - id: "3"
    content: Fix N+1 queries in task.rs::add_tags() - use single query with IN clause for tag verification
    status: pending
  - id: "4"
    content: Fix N+1 queries in goal.rs::add_tags() - use single query with IN clause for tag verification
    status: pending
  - id: "5"
    content: Fix string formatting in activity.rs::get_all() - use prepared statement with parameter
    status: pending
  - id: "6"
    content: Add LIMIT to entry.rs::find_all() (default 1000, make configurable if needed)
    status: pending
  - id: "7"
    content: Add LIMIT to goal.rs::find_all() (default 1000)
    status: pending
  - id: "8"
    content: Add LIMIT to tag.rs::find_all() (default 1000, though tags are typically small)
    status: pending
  - id: "9"
    content: Add LIMIT to activity.rs::get_by_entity() (default 1000)
    status: pending
  - id: "10"
    content: Fix transaction scope in goal.rs::get_or_create_current_instance() - move find_by_id() before transaction
    status: pending
---

# Rust libSQL Performance Audit and Optimization Plan

## Overview

This plan addresses performance issues found in the libSQL Rust codebase, focusing on SQLite optimizations, query patterns, and connection management.

## Issues Found

### Critical Issues

1. **Missing WAL Mode** (`desktop/src-tauri/src/db/connection.rs`)

- Current pragmas don't include `PRAGMA journal_mode = WAL`
- WAL is essential for concurrent read/write performance
- Should be the first pragma applied

2. **N+1 Query Patterns** (Multiple files)

- `entry.rs::add_tags()`: Loops through tag_ids, querying each tag individually
- `task.rs::add_tags()`: Same pattern
- `goal.rs::add_tags()`: Same pattern
- `search.rs::search_semantic()`: Loops through resource types, executing separate queries

3. **String Formatting in SQL** (`activity.rs::get_all()`)

- Uses `format!()` to build SQL with LIMIT
- Should use prepared statement with parameter

4. **Missing LIMITs on Unbounded Queries**

- `entry.rs::find_all()`: No LIMIT
- `goal.rs::find_all()`: No LIMIT
- `tag.rs::find_all()`: No LIMIT
- `activity.rs::get_by_entity()`: No LIMIT

### Optimization Opportunities

5. **Pragma Values** (`connection.rs::apply_sqlite_optimizations()`)

- `cache_size = -32000` (32MB) could be increased to -64000 (64MB)
- `mmap_size = 67108864` (64MB) could be increased to 268435456 (256MB)
- Missing `journal_mode = WAL` (critical)

6. **Transaction Scope** (`goal.rs::get_or_create_current_instance()`)

- Transaction includes call to `find_by_id()` which creates its own connection
- Should prepare data before starting transaction

7. **Bulk Operations** (Multiple files)

- Tag verification in loops could use single query with IN clause
- Bulk inserts already use transactions (good)

## Implementation Plan

### Phase 1: Critical Fixes

1. **Add WAL Mode to Pragmas** (`desktop/src-tauri/src/db/connection.rs`)

- Add `PRAGMA journal_mode = WAL` as first pragma
- Increase cache_size to -64000 (64MB)
- Increase mmap_size to 268435456 (256MB)

2. **Fix N+1 Queries in Tag Operations**

- `entry.rs::add_tags()`: Replace loop with single query using IN clause
- `task.rs::add_tags()`: Same fix
- `goal.rs::add_tags()`: Same fix

3. **Fix String Formatting in activity.rs**

- Replace `format!()` with prepared statement using `?1` parameter

### Phase 2: Query Optimizations

4. **Add LIMITs to Unbounded Queries**

- `entry.rs::find_all()`: Add LIMIT 1000 (or make configurable)
- `goal.rs::find_all()`: Add LIMIT 1000
- `tag.rs::find_all()`: Add LIMIT 1000 (tags are typically small)
- `activity.rs::get_by_entity()`: Add LIMIT 1000

5. **Optimize search_semantic in search.rs**

- Consider batching queries or using UNION ALL if possible
- At minimum, ensure connection is reused across loop iterations

### Phase 3: Transaction Improvements

6. **Fix Transaction Scope in goal.rs**

- Move `find_by_id()` call before transaction starts
- Keep transaction focused on database operations only

## Files to Modify

- `desktop/src-tauri/src/db/connection.rs` - Add WAL mode, optimize pragmas
- `desktop/src-tauri/src/db/repositories/entry.rs` - Fix N+1, add LIMIT
- `desktop/src-tauri/src/db/repositories/task.rs` - Fix N+1
- `desktop/src-tauri/src/db/repositories/goal.rs` - Fix N+1, fix transaction scope
- `desktop/src-tauri/src/db/repositories/tag.rs` - Add LIMIT
- `desktop/src-tauri/src/db/repositories/activity.rs` - Fix string formatting, add LIMIT
- `desktop/src-tauri/src/db/repositories/search.rs` - Optimize semantic search loop

## Expected Impact

- **WAL Mode**: 2-3x improvement in concurrent read/write performance
- **N+1 Fixes**: 10-100x improvement for operations with multiple tags
- **LIMITs**: Prevents memory issues and improves response times for large datasets
- **Pragma Optimizations**: 10-20% improvement in query performance
- **Prepared Statements**: Eliminates SQL injection risk, improves query caching

## Testing Considerations

- Verify WAL mode is active: `PRAGMA journal_mode;` should return `wal`
- Test tag operations with 10+ tags to verify N+1 fix
- Test queries with large datasets to verify LIMITs work
- Ensure all transactions still work correctly after scope changes