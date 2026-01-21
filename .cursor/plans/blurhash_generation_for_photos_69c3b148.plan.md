---
name: Blurhash generation for photos
overview: Add blurhash generation for all uploaded images using the blurhash crate. The blurhash will be generated automatically when images are saved through the unified media service and stored in the metadata JSON field. Also make the media service entity-agnostic to support entries, canvas, bookmarks, and tasks.
todos:
  - id: add-dependencies
    content: Add blurhash and image crates to Cargo.toml
    status: pending
  - id: create-migration
    content: Create database migration to make media_items entity-agnostic (entity_type, entity_id)
    status: pending
  - id: update-models
    content: Update MediaItem model to use entity_type and entity_id instead of entry_id
    status: pending
  - id: update-repository
    content: Update MediaRepository to support entity_type and entity_id
    status: pending
  - id: create-media-module
    content: Create media module with unified storage.rs and mod.rs
    status: pending
  - id: implement-blurhash
    content: Implement blurhash generation in save_media_file for image type
    status: pending
  - id: refactor-audio-storage
    content: Refactor audio/storage.rs to use unified save_media_file
    status: pending
  - id: update-lib-rs
    content: Update lib.rs to include media module and update ensure_media_directory calls
    status: pending
---

# Blurhash Generation for Photos

## Overview

Generate blurhash for every photo uploaded through the unified media service. The blurhash will be stored in the `metadata` JSON field of the `MediaItem` model, making it available for all resources that upload images. Additionally, make the media service entity-agnostic to support entries, canvas, bookmarks, and tasks.

## Architecture

The implementation will:

1. Add the `blurhash` crate dependency
2. Make media_items table entity-agnostic (support entries, canvas, bookmarks, tasks)
3. Create a unified media storage function that handles all media types and entity types
4. Generate blurhash when `media_type == "image"`
5. Store blurhash in the metadata JSON alongside other metadata

## Implementation Details

### 1. Add blurhash dependency

- Add `blurhash = "0.1"` to `[dependencies]` in `desktop/src-tauri/Cargo.toml`
- The blurhash crate requires an image decoding library. We'll use `image` crate for decoding images

### 2. Database migration for entity-agnostic media

- Create migration `009_make_media_entity_agnostic.sql`:
  - Add `entity_type` column (TEXT NOT NULL) with CHECK constraint: `entity_type IN ('entry', 'canvas', 'bookmark', 'task')`
  - Rename `entry_id` to `entity_id` (TEXT NOT NULL)
  - Remove foreign key constraint (can't have FK to multiple tables)
  - Migrate existing data: set `entity_type = 'entry'` for all existing rows
  - Update index: rename `idx_media_entry_id` to `idx_media_entity` and include both `entity_type` and `entity_id`
  - Add composite index on `(entity_type, entity_id)` for efficient lookups

### 3. Update MediaItem model

- Update `desktop/src-tauri/src/db/models.rs`:
  - Change `entry_id: String` to `entity_id: String`
  - Add `entity_type: String` field
  - Update serde attributes if needed

### 4. Update MediaRepository

- Update `desktop/src-tauri/src/db/repositories/media.rs`:
  - Change `create()` method signature to accept `entity_type: String` and `entity_id: String` instead of `entry_id: String`
  - Update `find_by_entry_id()` to `find_by_entity()` that takes `entity_type` and `entity_id`
  - Update all SQL queries to use `entity_type` and `entity_id`
  - Update `row_to_media_item()` to read new columns

### 5. Create unified media storage function

- Create `desktop/src-tauri/src/media/storage.rs` (new module) with a unified `save_media_file` function
- This function will replace/extend the current audio-specific storage
- Function signature: `pub async fn save_media_file(database: Arc<Database>, entity_type: String, entity_id: String, media_type: String, file_data: Vec<u8>, additional_metadata: serde_json::Value) -> Result<String>`

### 6. Blurhash generation

- When `media_type == "image"`, decode the image using the `image` crate
- Generate blurhash using `blurhash::encode()` with standard parameters (4x3 components)
- Add blurhash to the metadata JSON: `metadata["blurhash"] = blurhash_string`
- Handle errors gracefully - if blurhash generation fails, log a warning but don't fail the upload

### 7. Update audio storage to use unified function

- Refactor `desktop/src-tauri/src/audio/storage.rs::save_audio_file` to call the unified `save_media_file` function
- Update call to pass `entity_type: "entry"` and `entity_id` instead of just `entry_id`
- This maintains backward compatibility while using the unified service

### 8. Create media module

- Create `desktop/src-tauri/src/media/mod.rs` to export the storage functions
- Update `desktop/src-tauri/src/lib.rs` to include the media module
- Re-export `ensure_media_directory` from media module (currently in audio module)

### 9. Image decoding

- Add `image = "0.24"` to `[dependencies]` in `Cargo.toml` for image decoding
- Support common image formats: JPEG, PNG, WebP, GIF

### 10. Update schema.rs

- Update `desktop/src-tauri/src/db/schema.rs` to reflect new schema with `entity_type` and `entity_id`
- This ensures new databases get the correct schema

## Files to Modify

1. **desktop/src-tauri/Cargo.toml**

   - Add `blurhash = "0.1"` dependency
   - Add `image = "0.24"` dependency

2. **desktop/src-tauri/migrations/009_make_media_entity_agnostic.sql** (new file)

   - Migration to add `entity_type` column
   - Rename `entry_id` to `entity_id`
   - Remove foreign key constraint
   - Migrate existing data
   - Update indexes

3. **desktop/src-tauri/src/db/models.rs**

   - Update `MediaItem` struct: change `entry_id` to `entity_id` and add `entity_type`

4. **desktop/src-tauri/src/db/repositories/media.rs**

   - Update `create()` to accept `entity_type` and `entity_id`
   - Update `find_by_entry_id()` to `find_by_entity()` with both parameters
   - Update all SQL queries and row parsing

5. **desktop/src-tauri/src/db/schema.rs**

   - Update media_items table schema to use `entity_type` and `entity_id`
   - Remove foreign key constraint
   - Update indexes

6. **desktop/src-tauri/src/media/storage.rs** (new file)

   - Unified `save_media_file` function with `entity_type` and `entity_id` parameters
   - Blurhash generation logic for images
   - Helper functions for file extension detection

7. **desktop/src-tauri/src/media/mod.rs** (new file)

   - Module exports

8. **desktop/src-tauri/src/audio/storage.rs**

   - Refactor `save_audio_file` to use unified `save_media_file`
   - Update to pass `entity_type: "entry"` and `entity_id`
   - Keep audio-specific helper functions

9. **desktop/src-tauri/src/lib.rs**

   - Add `pub mod media;`
   - Update `ensure_media_directory` call to use `media::ensure_media_directory`

10. **desktop/src-tauri/src/audio/mod.rs**

    - Re-export storage functions or remove if moved to media module

11. **desktop/src-tauri/src/commands/audio.rs**

    - Update `get_media_items_for_entry` to use `find_by_entity("entry", entry_id)` instead of `find_by_entry_id`
    - Keep the command signature the same for backward compatibility (it's entry-specific)

## Blurhash Generation Logic

```rust
// Pseudocode for blurhash generation
if media_type == "image" {
    let img = image::load_from_memory(&file_data)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let pixels: Vec<[u8; 4]> = rgba.pixels().map(|p| p.0).collect();
    
    let blurhash = blurhash::encode(4, 3, width as usize, height as usize, &pixels)?;
    metadata["blurhash"] = serde_json::Value::String(blurhash);
}
```

## Error Handling

- If image decoding fails, log warning and continue without blurhash
- If blurhash generation fails, log warning and continue without blurhash
- Never fail the entire upload if blurhash generation fails

## Entity Types

The media service now supports the following entity types:
- `entry` - Journal entries
- `canvas` - Canvas items
- `bookmark` - Bookmarks
- `task` - Tasks

When uploading media, callers must specify both `entity_type` and `entity_id`:
```rust
save_media_file(
    database,
    "entry".to_string(),      // or "canvas", "bookmark", "task"
    entry_id,
    "image".to_string(),
    file_data,
    additional_metadata
).await?
```

## Future Image Uploads

When image upload commands are added (e.g., `save_image_file`), they should call the unified `save_media_file` function with the appropriate `entity_type` and `entity_id`, which will automatically generate blurhash for images.