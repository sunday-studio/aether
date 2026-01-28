---
name: "Rewrite Database Migrations"
overview: "Consolidate and rewrite all database migrations as if building the schema for the first time"
todos:
  - id: backup_migrations
    content: "Backup existing migrations directory"
    status: pending
  - id: delete_old_migrations
    content: "Delete all 15 existing migration files (001-015)"
    status: pending
  - id: create_001_initial
    content: "Create 001_initial_schema.sql with all core tables (settings, tags, entries, goals, tasks, subtasks, activities, bookmarks, canvases, resource links)"
    status: pending
  - id: create_002_search
    content: "Create 002_search_and_fts.sql with FTS5 indexes, mapping tables, and triggers for all resources"
    status: pending
  - id: create_003_media
    content: "Create 003_media_and_transcription.sql with entity-agnostic media_items and audio_transcriptions tables"
    status: pending
  - id: create_004_sync
    content: "Create 004_sync_infrastructure.sql with sync tables, columns on all tables, and sync triggers"
    status: pending
  - id: create_005_vectors
    content: "Create 005_vector_embeddings.sql with commented-out vector embedding structure"
    status: pending
  - id: update_migration_runner
    content: "Remove special handling for migration 004 (timezone migration) in migrations.rs"
    status: pending
  - id: test_migrations
    content: "Test all migrations on a fresh database and verify all tables, indexes, and triggers are created correctly"
    status: pending
isProject: false
---

# Database Migration Rewrite Plan

## Overview

Rewrite all 15 existing migrations into 5 clean, consolidated migrations that represent the database schema as if we're building it from scratch. This removes historical complexity and creates a cleaner foundation.

## Current State Analysis

**Existing migrations (15 files):**

- 001: Initial schema (core tables)
- 002: Add timestamps to goal_instances
- 003: Activities table
- 004: Media/transcription + settings migration
- 005: FTS5 search indexes
- 006: Vector embeddings (commented out)
- 007: Canvas tables
- 008: Bookmarks table
- 009: Make media entity-agnostic
- 010: Resource links
- 011: Fix FTS5 detail setting
- 012: Sync infrastructure tables
- 013: Sync columns on all tables
- 014: Sync triggers
- 015: Fix vector indexes (commented out)

**Issues to fix:**

- Settings table migration complexity (004)
- Media table needed refactoring (009)
- FTS5 had wrong detail setting, needed fix (011)
- Sync split across 3 migrations (012, 013, 014)
- Vector embeddings commented out in multiple places
- Core features (activities, bookmarks, canvases, resource links) split across multiple migrations

## New Migration Structure

### 001_initial_schema.sql

**Purpose:** All core application tables with complete structure from the start

**Contents:**

- Settings table (key-value store, correct structure from start)
- Tags table
- Entries table
- Entry-Tag junction table
- Goals table
- Goal-Tag junction table
- Goal Instances table (with updated_at and deleted_at from start)
- Goal Instance-Tag junction table
- Tasks table
- Task-Tag junction table
- Subtasks table
- Activities table (for activity tracking/audit logging)
- Bookmarks table
- Bookmark-Tag junction table
- Canvases table
- Resource links table (bidirectional linking)
- All indexes for core tables

**Key improvements:**

- Settings as key-value from the start (no migration needed)
- Goal instances have timestamps from the start
- All core features in one place
- All foreign keys and indexes defined properly

### 002_search_and_fts.sql

**Purpose:** Full-text search infrastructure with correct settings from the start

**Contents:**

- FTS5 virtual tables for: entries, tasks, subtasks, goals, tags, bookmarks
- FTS mapping tables for each resource type
- Insert/update/delete triggers for all FTS tables
- Backfill existing data into FTS indexes
- All with `detail='full'` from the start (no fix needed)

**Key improvements:**

- Correct FTS5 detail setting from the start
- All search infrastructure in one place
- Bookmarks FTS included here

### 003_media_and_transcription.sql

**Purpose:** Media handling with entity-agnostic structure from the start

**Contents:**

- Media items table (entity-agnostic from start: entity_type, entity_id)
- Audio transcriptions table
- All indexes and foreign keys
- No migration from entry-specific structure needed

**Key improvements:**

- Entity-agnostic structure from the start (no refactoring needed)
- Supports entries, canvases, bookmarks, tasks from day one

### 004_sync_infrastructure.sql

**Purpose:** Complete sync system in one migration

**Contents:**

- Sync infrastructure tables (_sync_outbox, _sync_meta, _sync_unknown)
- Sync columns on all tables (_sync_id, _updated_at, _deleted, _extra, _version)
- Unique indexes on _sync_id for all tables
- Sync triggers for all tables (insert, update, delete)
- Initialize _suppress_triggers flag

**Key improvements:**

- All sync functionality in one place
- No need to split across multiple migrations
- Triggers reference columns that exist (since columns are added first)

### 005_vector_embeddings.sql

**Purpose:** Vector embedding structure (commented out for future use)

**Contents:**

- Embedding columns for all searchable resources (commented out)
- Vector index creation statements (commented out)
- Clear comments explaining when/how to enable

**Key improvements:**

- Structure ready for future use
- Clear documentation on enabling

## Implementation Steps

1. **Backup existing migrations directory**

- Create backup of `desktop/src-tauri/migrations/`

2. **Delete old migration files**

- Remove all 15 existing migration files

3. **Create new migration files**

- Write 001_initial_schema.sql (includes all core tables: entries, tags, goals, tasks, subtasks, activities, bookmarks, canvases, resource links)
- Write 002_search_and_fts.sql
- Write 003_media_and_transcription.sql
- Write 004_sync_infrastructure.sql
- Write 005_vector_embeddings.sql

4. **Update migration runner if needed**

- Verify migration runner handles the new structure
- Remove special handling for migration 004 (timezone migration)

5. **Test migrations**

- Test on fresh database
- Verify all tables, indexes, triggers created correctly

## Files to Modify

- `desktop/src-tauri/migrations/001_initial_schema.sql` (new)
- `desktop/src-tauri/migrations/002_search_and_fts.sql` (new)
- `desktop/src-tauri/migrations/003_media_and_transcription.sql` (new)
- `desktop/src-tauri/migrations/004_sync_infrastructure.sql` (new)
- `desktop/src-tauri/migrations/005_vector_embeddings.sql` (new)
- `desktop/src-tauri/src/db/migrations.rs` (remove special handling for 004)

## Files to Delete

- All existing migration files (001-015)

## Notes

- Since the app is not in production, we can safely rewrite migrations
- The new structure is cleaner and easier to understand
- All functionality is preserved, just better organized
- Vector embeddings remain commented out but ready for future use
- Sync functionality fully preserved and consolidated
- Core features (activities, bookmarks, canvases, resource links) are part of the initial schema