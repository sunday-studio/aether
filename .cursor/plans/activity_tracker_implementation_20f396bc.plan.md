---
name: Activity Tracker Implementation
overview: "Implement a GitHub-style activity tracker that logs all user actions (create, update, delete) as activities. The system will serve dual purposes: generating activity counts for the frontend map visualization and providing a detailed audit log for backend operations."
todos: []
---

# Activity Tracker Implementation Plan

## Overview

This plan implements a GitHub-style activity tracker that combines activity counting for visualization with detailed audit logging. Every action (create, update, delete) will generate an activity record that can be aggregated by date for the frontend map.

## Architecture

The system will use a single `activities` table that stores:

- Action type (create, update, delete)
- Entity type (entry, task, subtask, goal, tag)
- Entity ID (for audit trail)
- Timestamp (for grouping by date)
- Optional metadata (JSON for future extensibility)

## Implementation Steps

### 1. Database Schema

**File**: `desktop/src-tauri/backend/migrations/003_create_activities_table.sql`

Create a new migration file with:

- `activities` table with columns: `id`, `action_type`, `entity_type`, `entity_id`, `created_at`, `metadata`
- Index on `created_at` for efficient date-based queries
- Index on `(entity_type, entity_id)` for audit log lookups

### 2. Activity Model

**File**: `desktop/src-tauri/backend/src/db/models.rs`

Add `Activity` struct with:

- `id: String`
- `action_type: String` (create, update, delete)
- `entity_type: String` (entry, task, subtask, goal, tag, goal_instance)
- `entity_id: String`
- `created_at: DateTime<Utc>`
- `metadata: Option<serde_json::Value>` (for future extensibility)

### 3. Activity Repository

**File**: `desktop/src-tauri/backend/src/db/repositories/activity.rs` (new file)

Create `ActivityRepository` with methods:

- `create()` - Insert new activity
- `get_by_date_range()` - Get activities grouped by date with detailed breakdowns
  - Returns: `HashMap<String, HashMap<String, HashMap<String, i64>>>` 
  - Structure: `date -> entity_type -> action_type -> count`
- `get_by_entity()` - Get audit log for specific entity
- `get_all()` - Get all activities (for audit purposes)

### 4. Activity Service/Helper

**File**: `desktop/src-tauri/backend/src/utils/activity.rs` (new file)

Create helper functions to log activities:

- `log_activity()` - Generic function to create activity records
- Helper macros/functions for common patterns (log_create, log_update, log_delete)

This will be called from handlers after successful operations.

### 5. Handler Integration

Update all handlers to log activities after successful operations:

**Files to update**:

- `desktop/src-tauri/backend/src/handlers/entry.rs` - Log on create, update, delete, add_tags, remove_tags
- `desktop/src-tauri/backend/src/handlers/task.rs` - Log on create, update, delete, add_tags, remove_tags, add_goal, remove_goal
  - **Special handling**: When `is_completed` changes from `false` to `true`, log as `complete` action (not `update`)
  - When `is_completed` changes from `true` to `false`, log as `update` action
- `desktop/src-tauri/backend/src/handlers/goal.rs` - Log on create, update, delete
- `desktop/src-tauri/backend/src/handlers/tag.rs` - Log on create
- `desktop/src-tauri/backend/src/handlers/trash.rs` - Log on restore

For subtasks, log in task handlers when subtask operations occur:

- `create_subtask` - Log as `subtask` entity with `create` action
- `update_subtask` - Log as `subtask` entity with `update` action (check for completion like tasks)
- `delete_subtask` - Log as `subtask` entity with `delete` action
- `reorder_subtasks` - Log as `subtask` entity with `reorder` action

### 6. API Endpoint

**File**: `desktop/src-tauri/backend/src/handlers/activity.rs` (new file)

Create new handler with endpoint:

- `GET /v1/activities` - Returns detailed activity breakdowns grouped by date
  - Query params: `start_date`, `end_date` (optional, defaults to last year)
  - Response format:
    ```json
    {
      "2024-01-15": {
        "entry": { "create": 6, "update": 5 },
        "goal": { "create": 5 },
        "task": { "create": 6, "complete": 5 },
        "subtask": { "create": 6 }
      },
      "2024-01-16": {
        "entry": { "create": 2, "update": 1 },
        "task": { "create": 3, "complete": 2 }
      }
    }
    ```

  - Each date contains a breakdown by entity type, then by action type
  - Action types: `create`, `update`, `delete`, `complete` (for tasks), `add_tags`, `remove_tags`, `add_goal`, `remove_goal`, `reorder` (for subtasks), `restore` (for trash)

**File**: `desktop/src-tauri/backend/src/api/routes.rs`

Add route: `.route("/v1/activities", get(activity::get_activities))`

### 7. OpenAPI Documentation

**File**: `desktop/src-tauri/backend/src/api/openapi.rs`

Add activity endpoint to OpenAPI spec with proper response schema.

## Data Flow

```
User Action → Handler → Repository Operation → Activity Logging → Database
                                                      ↓
Frontend Request → GET /v1/activities → Aggregate by Date/Entity/Action → Return Detailed Breakdowns
```

## Activity Types

Track these actions:

- **Entry**: create, update, delete, add_tags, remove_tags
- **Task**: create, update, delete, complete (when is_completed changes to true), add_tags, remove_tags, add_goal, remove_goal
- **SubTask**: create, update, delete, complete (when is_completed changes to true), reorder
- **Goal**: create, update, delete
- **Tag**: create
- **Trash**: restore_task

**Important**: The `complete` action is a special case for tasks and subtasks. It should only be logged when `is_completed` transitions from `false` to `true`. Regular updates to other fields should use the `update` action.

## Notes

- Activities are append-only (no updates/deletes)
- Use transactions to ensure activity logging happens atomically with the main operation
- Consider performance: activity logging should be fast (async, non-blocking)
- The metadata field allows future extensibility without schema changes
- Date grouping should use UTC dates to ensure consistency across timezones
- For task/subtask completion tracking, handlers need to check the previous state to determine if it's a completion (false → true) or just an update
- The API response structure allows frontend to display messages like "6 new entries", "update 5 entries", "completed 5 tasks", etc.