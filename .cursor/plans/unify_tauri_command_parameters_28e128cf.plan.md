---
name: Unify Tauri Command Parameters
overview: "Unify all Tauri commands to use three standardized parameters: requestData (request body), queryParams (query parameters), and pathParams (path parameters). This eliminates inconsistencies and simplifies the API client logic."
todos:
  - id: create-helper-types
    content: Create helper parameter types in desktop/src-tauri/src/commands/params.rs (EmptyPathParams, IdPathParams, EmptyQueryParams, etc.)
    status: pending
  - id: simplify-api-client
    content: Standardize API client in desktop/src/lib/api-client.ts to always pass request_data, query_params, and path_params - remove all complex detection logic (lines 218-265)
    status: pending
  - id: migrate-sync-commands
    content: Migrate sync commands (configure_sync, reconnect_sync, ensure_media_blob) to unified parameter pattern
    status: pending
  - id: migrate-entry-commands
    content: Migrate entry commands (8 commands) - get_entries, get_entry_by_id, create_entry, bulk_create_entries, update_entry, delete_entry, add_tags_to_entry, remove_tags_from_entry
    status: pending
  - id: migrate-task-commands
    content: Migrate task commands (15 commands) - create_task, get_inbox_tasks, get_overdue_tasks, get_task_by_id, update_task, delete_task, get_subtasks, create_subtask, update_subtask, delete_subtask, reorder_subtasks, add_tags_to_task, remove_tags_from_task, add_goal_to_task, remove_goal_from_task
    status: pending
  - id: migrate-goal-commands
    content: Migrate goal commands (10 commands) - get_goals, get_goal_by_id, create_goal, update_goal, delete_goal, get_goal_instances, get_current_goal_instance, add_tags_to_goal, remove_tags_from_goal
    status: pending
  - id: migrate-tag-commands
    content: Migrate tag commands (3 commands) - get_all_tags, create_tag, bulk_create_tags
    status: pending
  - id: migrate-bookmark-commands
    content: Migrate bookmark commands (9 commands) - get_bookmarks, get_bookmark_by_id, create_bookmark, update_bookmark, delete_bookmark, add_tags_to_bookmark, remove_tags_from_bookmark, extract_metadata
    status: pending
  - id: migrate-link-commands
    content: Migrate link commands (7 commands) - create_link, get_backlinks, get_outgoing_links, delete_link, search_linkable_resources, get_all_links_for_graph, sync_links_from_content
    status: pending
  - id: migrate-canvas-commands
    content: Migrate canvas commands (5 commands) - get_canvases, get_canvas_by_id, create_canvas, update_canvas, delete_canvas
    status: pending
  - id: migrate-audio-commands
    content: Migrate audio commands (5 commands) - save_audio_recording, get_audio_data, delete_audio_recording, get_media_items_for_entry, get_audio_metadata
    status: pending
  - id: migrate-transcription-commands
    content: Migrate transcription commands (10 commands) - start_transcription, get_transcriptions, get_transcription_by_id, set_active_transcription, list_providers, validate_provider, list_available_models, download_model, verify_model, delete_model
    status: pending
  - id: migrate-settings-commands
    content: Migrate settings commands (3 commands) - get_setting, get_all_settings, set_setting
    status: pending
  - id: migrate-search-commands
    content: Migrate search commands (1 command) - search_resources
    status: pending
  - id: migrate-activity-commands
    content: Migrate activity commands (1 command) - get_activities
    status: pending
  - id: migrate-trash-commands
    content: Migrate trash commands (2 commands) - get_trashed_tasks, restore_task
    status: pending
  - id: migrate-embeddings-commands
    content: Migrate embeddings commands (4 commands) - list_embedding_models, download_embedding_model, verify_embedding_model, delete_embedding_model
    status: pending
  - id: add-cursor-rule
    content: Create .cursor/rules/tauri-commands.md documenting the unified parameter pattern
    status: pending
  - id: test-all-commands
    content: Test all command types (request_data only, path_params only, query_params only, combinations) to ensure they work correctly
    status: pending
isProject: false
---

# Unify Tauri Command Parameters

## Problem Analysis

Current commands have inconsistent parameter patterns:

- Some take `request: RequestStruct` 
- Some take individual fields like `document: String, date: Option<...>`
- Some take `payload: Vec<T>`
- Path params are passed as individual parameters
- Query params are passed as individual parameters

This causes errors like "command configure_sync missing required key request" and makes the API client logic complex and error-prone.

## Unified Solution

**All commands will use three standardized parameters:**

1. **`request_data: Option<RequestStruct>`** - Request body data (POST/PUT bodies)
2. **`query_params: Option<QueryParamsStruct>`** - Query parameters (from URL ?key=value)
3. **`path_params: Option<PathParamsStruct>`** - Path parameters (from URL /:id)

### Benefits

- **Consistency**: All commands follow the same pattern
- **Simplicity**: API client always passes the same three parameters
- **Type Safety**: Strongly typed structs for each parameter type
- **Maintainability**: Easy to understand and modify

## Implementation Strategy

### Phase 1: Define Standard Parameter Types

Create helper types/macros for common patterns:

```rust
// For commands with no path params
#[derive(Deserialize)]
pub struct EmptyPathParams {}

// For commands with path params like /:id
#[derive(Deserialize)]
pub struct IdPathParams {
    pub id: String,
}

// For commands with query params
#[derive(Deserialize)]
pub struct SearchQueryParams {
    pub q: String,
    pub types: Option<String>,
    pub limit: Option<u32>,
    // ...
}
```

### Phase 2: Update Command Signatures

**Before:**

```rust
#[tauri::command]
pub async fn configure_sync(
    app: AppHandle,
    engine: State<'_, Arc<SyncEngine>>,
    request: ConfigureSyncRequest,
) -> Result<()>
```

**After:**

```rust
#[tauri::command]
pub async fn configure_sync(
    app: AppHandle,
    engine: State<'_, Arc<SyncEngine>>,
    request_data: Option<ConfigureSyncRequest>,
    query_params: Option<EmptyQueryParams>,
    path_params: Option<EmptyPathParams>,
) -> Result<()> {
    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    // ... rest of logic
}
```

**For commands with path params:**

```rust
#[tauri::command]
pub async fn get_entry_by_id(
    state: State<'_, DbState>,
    request_data: Option<EmptyRequest>,
    query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<Entry> {
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    // ... rest of logic
}
```

### Phase 3: Standardize API Client

**File**: `desktop/src/lib/api-client.ts`

**Current Issues:**

- Complex body handling logic (lines 236-265) tries to detect different patterns
- Inconsistent parameter passing (sometimes `request`, sometimes individual fields, sometimes `payload`)
- Query params merged directly into args (lines 222-234)
- Path params merged directly into args (line 218)
- Numeric conversion logic for specific params (lines 221-230)

**New Standardized Approach:**

The API client will always pass three standardized parameters:

```typescript
// Simplified argument preparation
const args: Record<string, unknown> = {
    request_data: body?.data ?? null,
    query_params: Object.keys(match.queryParams).length > 0 ? match.queryParams : null,
    path_params: Object.keys(match.params).length > 0 ? match.params : null,
};

// Remove null values to keep payload clean
if (args.request_data === null) delete args.request_data;
if (args.query_params === null) delete args.query_params;
if (args.path_params === null) delete args.path_params;
```

**Changes Required:**

1. Remove all complex body detection logic (lines 236-265)
2. Remove query param merging logic (lines 219-235)
3. Remove path param merging logic (line 218)
4. Always pass `request_data`, `query_params`, `path_params` as three separate parameters
5. Handle numeric conversion in Rust side if needed, or pass query params as strings and let Rust deserialize
6. Simplify the entire `customFetch` function significantly

### Phase 4: Complete Command Migration

**All Commands to Migrate (87 total):**

#### Sync Commands (6 commands) - `desktop/src-tauri/src/commands/sync.rs`

- `configure_sync` - POST with request body
- `sync_now` - POST with no params
- `get_sync_status` - GET with no params
- `disconnect_sync` - POST with no params
- `reconnect_sync` - POST with request body
- `ensure_media_blob` - POST with path param (mediaId)

#### Entry Commands (8 commands) - `desktop/src-tauri/src/commands/entry.rs`

- `get_entries` - GET with no params
- `get_entry_by_id` - GET with path param (id)
- `create_entry` - POST with request body (individual fields)
- `bulk_create_entries` - POST with array payload
- `update_entry` - PUT with path param (id) + request body
- `delete_entry` - DELETE with path param (id)
- `add_tags_to_entry` - POST with path param (id) + array body
- `remove_tags_from_entry` - DELETE with path param (id) + string body

#### Task Commands (15 commands) - `desktop/src-tauri/src/commands/task.rs`

- `create_task` - POST with request body (individual fields)
- `get_inbox_tasks` - GET with no params
- `get_overdue_tasks` - GET with no params
- `get_task_by_id` - GET with path param (id)
- `update_task` - PUT with path param (id) + request body
- `delete_task` - DELETE with path param (id)
- `get_subtasks` - GET with path param (taskId)
- `create_subtask` - POST with path param (taskId) + request body
- `update_subtask` - PUT with path params (taskId, subtaskId) + request body
- `delete_subtask` - DELETE with path params (taskId, subtaskId)
- `reorder_subtasks` - POST with path param (taskId) + request body
- `add_tags_to_task` - POST with path param (id) + array body
- `remove_tags_from_task` - DELETE with path param (id) + array body
- `add_goal_to_task` - POST with path param (id) + request body
- `remove_goal_from_task` - DELETE with path param (id)

#### Goal Commands (10 commands) - `desktop/src-tauri/src/commands/goal.rs`

- `get_goals` - GET with no params
- `get_goal_by_id` - GET with path param (id)
- `create_goal` - POST with request body (individual fields)
- `update_goal` - PUT with path param (id) + request body
- `delete_goal` - DELETE with path param (id)
- `get_goal_instances` - GET with path param (goalId)
- `get_current_goal_instance` - GET with path param (goalId)
- `add_tags_to_goal` - POST with path param (id) + array body
- `remove_tags_from_goal` - DELETE with path param (id) + array body

#### Tag Commands (3 commands) - `desktop/src-tauri/src/commands/tag.rs`

- `get_all_tags` - GET with no params
- `create_tag` - POST with request body (individual fields)
- `bulk_create_tags` - POST with array payload

#### Bookmark Commands (9 commands) - `desktop/src-tauri/src/commands/bookmark.rs`

- `get_bookmarks` - GET with query params
- `get_bookmark_by_id` - GET with path param (id)
- `create_bookmark` - POST with request body (individual fields)
- `update_bookmark` - PUT with path param (id) + request body
- `delete_bookmark` - DELETE with path param (id)
- `add_tags_to_bookmark` - POST with path param (id) + array body
- `remove_tags_from_bookmark` - DELETE with path param (id) + array body
- `extract_metadata` - GET with query param (url)

#### Link Commands (7 commands) - `desktop/src-tauri/src/commands/link.rs`

- `create_link` - POST with request body (individual fields)
- `get_backlinks` - GET with query params
- `get_outgoing_links` - GET with query params
- `delete_link` - DELETE with query params
- `search_linkable_resources` - GET with query params
- `get_all_links_for_graph` - GET with no params
- `sync_links_from_content` - POST with request body

#### Canvas Commands (5 commands) - `desktop/src-tauri/src/commands/canvas.rs`

- `get_canvases` - GET with no params
- `get_canvas_by_id` - GET with path param (id)
- `create_canvas` - POST with request body (individual fields)
- `update_canvas` - PUT with path param (id) + request body
- `delete_canvas` - DELETE with path param (id)

#### Audio Commands (5 commands) - `desktop/src-tauri/src/commands/audio.rs`

- `save_audio_recording` - POST with request body
- `get_audio_data` - GET with path param (mediaId)
- `delete_audio_recording` - DELETE with path param (mediaId)
- `get_media_items_for_entry` - GET with path param (entryId)
- `get_audio_metadata` - GET with path param (mediaId)

#### Transcription Commands (10 commands) - `desktop/src-tauri/src/commands/transcription.rs`

- `start_transcription` - POST with path param (mediaId) + optional query param
- `get_transcriptions` - GET with path param (mediaId)
- `get_transcription_by_id` - GET with path param (transcriptionId)
- `set_active_transcription` - POST with request body (transcriptionId, mediaId)
- `list_providers` - GET with no params
- `validate_provider` - POST with request body (provider_name)
- `list_available_models` - GET with no params
- `download_model` - POST with path param (modelSize)
- `verify_model` - POST with path param (modelSize)
- `delete_model` - DELETE with path param (modelSize)

#### Settings Commands (3 commands) - `desktop/src-tauri/src/commands/settings.rs`

- `get_setting` - GET with query param (key)
- `get_all_settings` - GET with no params
- `set_setting` - POST with request body (key, value as individual fields)

#### Search Commands (1 command) - `desktop/src-tauri/src/commands/search.rs`

- `search_resources` - GET with query params (q, types, tags, limit, offset, mode)

#### Activity Commands (1 command) - `desktop/src-tauri/src/commands/activity.rs`

- `get_activities` - GET with query params (start_date, end_date)

#### Trash Commands (2 commands) - `desktop/src-tauri/src/commands/trash.rs`

- `get_trashed_tasks` - GET with no params
- `restore_task` - POST with path param (id)

#### Embeddings Commands (4 commands) - `desktop/src-tauri/src/commands/embeddings.rs`

- `list_embedding_models` - GET with no params
- `download_embedding_model` - POST with path param (model_name)
- `verify_embedding_model` - POST with path param (model_name)
- `delete_embedding_model` - DELETE with path param (model_name)

### Migration Strategy

1. **Create helper types** for common parameter patterns (EmptyPathParams, IdPathParams, etc.)
2. **Standardize API client first** - This enables incremental migration
3. **Migrate commands module by module**:

   - Start with sync commands (fixes immediate issue)
   - Then entry commands
   - Then task commands
   - Continue through all modules in order

4. **Test each module** after migration before moving to next

### Phase 5: Add Validation Rule

Create `.cursor/rules/tauri-commands.md` documenting:

- All commands must use `request_data`, `query_params`, `path_params`
- How to define parameter structs
- Examples of each pattern
- Common mistakes to avoid

## Command Patterns

### Pattern 1: POST with Request Body Only

```rust
request_data: Option<CreateEntryRequest>
query_params: Option<EmptyQueryParams>
path_params: Option<EmptyPathParams>
```

### Pattern 2: GET with Path Parameter

```rust
request_data: Option<EmptyRequest>
query_params: Option<EmptyQueryParams>
path_params: Option<IdPathParams>  // { id: String }
```

### Pattern 3: GET with Query Parameters

```rust
request_data: Option<EmptyRequest>
query_params: Option<SearchQueryParams>  // { q: String, types: Option<String>, ... }
path_params: Option<EmptyPathParams>
```

### Pattern 4: POST with Path + Request Body

```rust
request_data: Option<AddTagsRequest>
query_params: Option<EmptyQueryParams>
path_params: Option<IdPathParams>  // { id: String }
```

## Files to Modify

1. **API Client**: `desktop/src/lib/api-client.ts` - Simplify to always pass three params
2. **Sync Commands**: `desktop/src-tauri/src/commands/sync.rs` - Migrate to unified pattern
3. **All Command Files**: Migrate all commands in `desktop/src-tauri/src/commands/`
4. **Helper Types**: Create `desktop/src-tauri/src/commands/params.rs` for common parameter structs
5. **Cursor Rule**: `.cursor/rules/tauri-commands.md` - Add standardization documentation

## Migration Order

1. **Create helper parameter types** (`desktop/src-tauri/src/commands/params.rs`)

   - EmptyPathParams, EmptyQueryParams, EmptyRequest
   - IdPathParams, MediaIdPathParams, TaskIdPathParams, etc.
   - Common query param structs (SearchQueryParams, etc.)

2. **Standardize API client** (`desktop/src/lib/api-client.ts`)

   - Simplify to always pass request_data, query_params, path_params
   - Remove all complex detection logic
   - This allows incremental migration (old and new commands can coexist temporarily)

3. **Migrate sync commands** (`desktop/src-tauri/src/commands/sync.rs`) - 6 commands

   - Fixes immediate issue with configure_sync
   - Test thoroughly before proceeding

4. **Migrate entry commands** (`desktop/src-tauri/src/commands/entry.rs`) - 8 commands

   - Test all entry operations

5. **Migrate task commands** (`desktop/src-tauri/src/commands/task.rs`) - 15 commands

   - Largest module, test carefully

6. **Migrate goal commands** (`desktop/src-tauri/src/commands/goal.rs`) - 10 commands

7. **Migrate tag commands** (`desktop/src-tauri/src/commands/tag.rs`) - 3 commands

8. **Migrate bookmark commands** (`desktop/src-tauri/src/commands/bookmark.rs`) - 9 commands

9. **Migrate link commands** (`desktop/src-tauri/src/commands/link.rs`) - 7 commands

10. **Migrate canvas commands** (`desktop/src-tauri/src/commands/canvas.rs`) - 5 commands

11. **Migrate audio commands** (`desktop/src-tauri/src/commands/audio.rs`) - 5 commands

12. **Migrate transcription commands** (`desktop/src-tauri/src/commands/transcription.rs`) - 10 commands

13. **Migrate settings commands** (`desktop/src-tauri/src/commands/settings.rs`) - 3 commands

14. **Migrate search commands** (`desktop/src-tauri/src/commands/search.rs`) - 1 command

15. **Migrate activity commands** (`desktop/src-tauri/src/commands/activity.rs`) - 1 command

16. **Migrate trash commands** (`desktop/src-tauri/src/commands/trash.rs`) - 2 commands

17. **Migrate embeddings commands** (`desktop/src-tauri/src/commands/embeddings.rs`) - 4 commands

18. **Add Cursor rule** (`.cursor/rules/tauri-commands.md`)

19. **Final testing** - Test all 87 endpoints comprehensively

## Testing Strategy

- Test commands with only request_data
- Test commands with only path_params
- Test commands with only query_params
- Test commands with combinations
- Test edge cases (empty params, missing required params)
- Test configure_sync specifically (the original issue)