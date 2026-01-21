---
name: Unify Model Managers
overview: Unify the transcription and embedding model managers by extracting common functionality into a shared module, while keeping separate managers. Add full download/verify/delete support for embedding models to match transcription models.
todos:
  - id: create_shared_models_module
    content: Create utils/models module with mod.rs, common.rs, download.rs, verify.rs, and types.rs
    status: pending
  - id: extract_common_directory_logic
    content: Extract platform-specific directory logic to common.rs
    status: pending
  - id: extract_download_logic
    content: Extract download functionality to download.rs with progress support
    status: pending
  - id: extract_verification_logic
    content: Extract verification logic to verify.rs with size and checksum support
    status: pending
  - id: refactor_transcription_manager
    content: Refactor transcription model_manager.rs to use shared utilities
    status: pending
  - id: enhance_embedding_manager
    content: Add download, verify, and delete functions to embedding model_manager.rs using shared utilities
    status: pending
  - id: add_embedding_commands
    content: Create Tauri commands for embedding model management (list, download, verify, delete)
    status: pending
  - id: register_commands
    content: Register new embedding model commands in lib.rs
    status: pending
  - id: update_exports
    content: Update module exports in utils/mod.rs and embeddings/mod.rs
    status: pending
---

# Unify Model Managers

## Current State Analysis

Two separate model managers exist with duplicated functionality:

1. **Transcription Model Manager** (`desktop/src-tauri/src/transcription/model_manager.rs`):

   - Full CRUD operations: list, download, verify, delete
   - Database integration for settings
   - Platform-specific directory logic
   - ModelInfo with: name, size, file_size, download_url, checksum, is_downloaded

2. **Embedding Model Manager** (`desktop/src-tauri/src/utils/embeddings/model_manager.rs`):

   - Basic operations: list, check if downloaded, ensure directory
   - Missing: download, verify, delete
   - Platform-specific directory logic (duplicated)
   - ModelInfo with: name, size, dimensions, file_size, downloaded

## Unification Strategy

### 1. Create Shared Model Infrastructure

Create `desktop/src-tauri/src/utils/models/` module with:

- **`common.rs`**: Shared platform-specific directory logic
  - `get_models_base_dir() -> Result<PathBuf>` - unified platform detection
  - `ensure_models_dir(category: &str) -> Result<()>` - create category subdirectories

- **`download.rs`**: Shared download functionality
  - `download_file(url: &str, path: &Path, progress_callback: Option<...>) -> Result<PathBuf>`
  - Generic download with progress tracking

- **`verify.rs`**: Shared verification logic
  - `verify_file(path: &Path, expected_size: Option<u64>, checksum: Option<&str>) -> Result<bool>`
  - File size and checksum verification

- **`types.rs`**: Shared types
  - Unified `ModelInfo` structure (with optional fields for model-specific data)
  - `ModelCategory` enum (Transcription, Embedding)

### 2. Refactor Transcription Model Manager

Update `desktop/src-tauri/src/transcription/model_manager.rs`:

- Use shared `get_models_base_dir()` from common module
- Use shared download/verify functions
- Keep transcription-specific logic (database settings, model URLs)
- Update ModelInfo to use unified structure

### 3. Enhance Embedding Model Manager

Update `desktop/src-tauri/src/utils/embeddings/model_manager.rs`:

- Add `download_model()` function (similar to transcription)
- Add `verify_model()` function
- Add `delete_model()` function
- Use shared common utilities
- Add download URLs to ModelInfo

### 4. Add Tauri Commands for Embedding Models

Create or update commands in `desktop/src-tauri/src/commands/`:

- `list_embedding_models()` - list available embedding models
- `download_embedding_model()` - download an embedding model
- `verify_embedding_model()` - verify embedding model integrity
- `delete_embedding_model()` - delete an embedding model

Register these commands in `desktop/src-tauri/src/lib.rs`

### 5. Unified ModelInfo Structure

Create a flexible ModelInfo that supports both use cases:

```rust
pub struct ModelInfo {
    pub name: String,
    pub size: String,
    pub file_size: u64,
    pub download_url: Option<String>,  // Some for transcription, None for embeddings initially
    pub checksum: Option<String>,
    pub is_downloaded: bool,
    // Model-specific fields
    pub dimensions: Option<u32>,  // Some for embeddings
    pub category: ModelCategory,
}
```

## File Changes

### New Files

- `desktop/src-tauri/src/utils/models/mod.rs`
- `desktop/src-tauri/src/utils/models/common.rs`
- `desktop/src-tauri/src/utils/models/download.rs`
- `desktop/src-tauri/src/utils/models/verify.rs`
- `desktop/src-tauri/src/utils/models/types.rs`

### Modified Files

- `desktop/src-tauri/src/transcription/model_manager.rs` - use shared utilities
- `desktop/src-tauri/src/utils/embeddings/model_manager.rs` - add full CRUD, use shared utilities
- `desktop/src-tauri/src/utils/embeddings/mod.rs` - export new functions
- `desktop/src-tauri/src/commands/transcription.rs` - potentially update if ModelInfo changes
- `desktop/src-tauri/src/lib.rs` - register new embedding model commands
- `desktop/src-tauri/src/utils/mod.rs` - add models module

## Implementation Details

### Shared Directory Logic

Both managers currently duplicate platform detection. Extract to:

- Single `get_models_base_dir()` function
- Category-based subdirectories: `models/transcription/` and `models/embeddings/`

### Download Functionality

Transcription has async download with progress. Extract to reusable function that:

- Accepts URL, destination path, optional progress callback
- Handles streaming, error handling, file creation
- Can be used by both managers

### Verification

Transcription has basic size verification. Enhance to:

- Support optional checksum verification
- Support optional size validation
- Reusable for both model types

### Database Integration

Transcription uses database for settings. Keep this in transcription manager only (it's specific to that use case), but make the pattern available if embeddings need it later.

## Benefits

1. **DRY Principle**: Eliminate duplicated platform detection and directory logic
2. **Consistency**: Both model types have same capabilities
3. **Maintainability**: Changes to common functionality happen in one place
4. **Extensibility**: Easy to add new model types in the future
5. **Feature Parity**: Embedding models get full management capabilities