# Tauri Command Parameter Standardization

## Overview

All Tauri commands must use a unified three-parameter pattern for consistency, type safety, and maintainability.

## Standard Parameter Pattern

Every Tauri command must accept exactly three parameters:

1. **`request_data: Option<RequestStruct>`** - Request body data (POST/PUT bodies)
2. **`query_params: Option<QueryParamsStruct>`** - Query parameters (from URL ?key=value)
3. **`path_params: Option<PathParamsStruct>`** - Path parameters (from URL /:id)

### Example

```rust
#[tauri::command]
pub async fn get_entry_by_id(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<Entry> {
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    // ... rest of logic
}
```

## Helper Parameter Types

Common parameter types are defined in `desktop/src-tauri/src/commands/params.rs`:

### Empty Types
- `EmptyPathParams` - For commands with no path parameters
- `EmptyQueryParams` - For commands with no query parameters
- `EmptyRequest` - For commands with no request body

### Path Parameter Types
- `IdPathParams` - Single ID parameter (`{ id: String }`)
- `MediaIdPathParams` - Media ID parameter (`{ media_id: String }`)
- `TaskIdPathParams` - Task ID parameter (`{ task_id: String }`)
- `EntryIdPathParams` - Entry ID parameter (`{ entry_id: String }`)
- `GoalIdPathParams` - Goal ID parameter (`{ goal_id: String }`)
- `TranscriptionIdPathParams` - Transcription ID parameter (`{ transcription_id: String }`)
- `TaskSubtaskPathParams` - Task and subtask IDs (`{ task_id: String, subtask_id: String }`)
- `ModelSizePathParams` - Model size parameter (`{ model_size: String }`)
- `ModelNamePathParams` - Model name parameter (`{ model_name: String }`)

### Query Parameter Types
- `SearchQueryParams` - Search query parameters
- `BookmarkQueryParams` - Bookmark listing filters
- `LinkQueryParams` - Link query parameters
- `ActivityQueryParams` - Activity date range filters
- `ExtractMetadataQueryParams` - Metadata extraction URL
- `SettingQueryParams` - Setting key parameter
- `TranscriptionStartQueryParams` - Transcription provider option

## Command Patterns

### Pattern 1: POST with Request Body Only

```rust
#[tauri::command]
pub async fn create_entry(
    state: State<'_, DbState>,
    request_data: Option<CreateEntryRequest>,
    _query_params: Option<EmptyQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<Entry> {
    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    // ... use request fields
}
```

### Pattern 2: GET with Path Parameter

```rust
#[tauri::command]
pub async fn get_entry_by_id(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<Entry> {
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    // ... use id
}
```

### Pattern 3: GET with Query Parameters

```rust
#[tauri::command]
pub async fn search_resources(
    state: State<'_, DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<SearchQueryParams>,
    _path_params: Option<EmptyPathParams>,
) -> Result<SearchResponse> {
    let params = query_params.ok_or_else(|| AppError::BadRequest("Query parameters are required".to_string()))?;
    // ... use params.q, params.types, etc.
}
```

### Pattern 4: POST with Path + Request Body

```rust
#[tauri::command]
pub async fn update_entry(
    state: State<'_, DbState>,
    request_data: Option<UpdateEntryRequest>,
    _query_params: Option<EmptyQueryParams>,
    path_params: Option<IdPathParams>,
) -> Result<Entry> {
    let id = path_params
        .and_then(|p| Some(p.id))
        .ok_or_else(|| AppError::BadRequest("ID is required".to_string()))?;
    let request = request_data.ok_or_else(|| AppError::BadRequest("Request data is required".to_string()))?;
    // ... use id and request fields
}
```

### Pattern 5: POST with Path + Query Parameters

```rust
#[tauri::command]
pub async fn start_transcription(
    app: AppHandle,
    state: State<'_, crate::DbState>,
    _request_data: Option<EmptyRequest>,
    query_params: Option<TranscriptionStartQueryParams>,
    path_params: Option<MediaIdPathParams>,
) -> Result<String> {
    let media_id = path_params
        .and_then(|p| Some(p.media_id))
        .ok_or_else(|| AppError::BadRequest("Media ID is required".to_string()))?;
    // ... use media_id and optional query params
}
```

## API Client Integration

The API client (`desktop/src/lib/api-client.ts`) automatically converts HTTP requests to the unified parameter pattern:

- Request body → `request_data`
- Query string parameters → `query_params`
- URL path parameters → `path_params`

No changes needed in frontend code that uses the SDK hooks (they go through the API client).

## Direct Invoke Calls

When calling Tauri commands directly with `invoke()`, use the unified pattern:

```typescript
// Before
await invoke("create_entry", {
  document: "...",
  date: "...",
});

// After
await invoke("create_entry", {
  request_data: {
    document: "...",
    date: "...",
  },
});
```

```typescript
// Before
await invoke("get_entry_by_id", { id: "123" });

// After
await invoke("get_entry_by_id", {
  path_params: { id: "123" },
});
```

## Naming Conventions

- Path parameter structs use `camelCase` field names (e.g., `media_id` becomes `mediaId` in JSON)
- Use `#[serde(rename_all = "camelCase")]` on path param structs
- Request structs should match the API schema naming
- Query param structs should match URL parameter names

## Common Mistakes to Avoid

1. **Don't use individual parameters** - Always use the three-parameter pattern
2. **Don't skip unused parameters** - Use `_request_data`, `_query_params`, `_path_params` for unused ones
3. **Don't mix patterns** - Don't have some commands with individual params and others with the unified pattern
4. **Don't forget validation** - Always validate required parameters with `ok_or_else()`
5. **Don't use snake_case in path params** - Path params come from URLs which use camelCase (e.g., `:mediaId`)

## Benefits

- **Consistency**: All commands follow the same pattern
- **Simplicity**: API client always passes the same three parameters
- **Type Safety**: Strongly typed structs for each parameter type
- **Maintainability**: Easy to understand and modify
- **Testability**: Predictable parameter structure
