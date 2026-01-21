---
name: Audio Transcription Backend Plan
overview: Implement the Rust backend for audio recording and transcription system, including database schema, media storage, transcription providers, settings management with encryption, and Tauri commands following existing codebase patterns.
todos:
  - id: phase1-database
    content: "Phase 1: Create database migration (004_add_audio_transcription_tables.sql) with media_items and audio_transcriptions tables. Migrate existing settings table to key-value structure (backup timezone, recreate table, restore timezone). Update schema.rs to create new settings table structure. Add models (MediaItem, AudioTranscription) and update Settings model to key-value structure in db/models.rs. Create MediaRepository, TranscriptionRepository, and SettingsRepository following existing repository patterns."
    status: completed
  - id: phase2-audio-storage
    content: "Phase 2: Implement audio storage module (audio/storage.rs, audio/recorder.rs) with filesystem storage. Functions: save_audio_file() (write to filesystem + DB), get_audio_file_path(), read_audio_file(), delete_audio() (delete file + DB). Platform-specific media directory management. Use MediaRepository for database operations."
    status: completed
  - id: phase3-settings-encryption
    content: "Phase 3: Implement settings management with encryption (settings/mod.rs, settings/encryption.rs). Add AES-256-GCM encryption for sensitive keys, OS keychain integration, and automatic encryption detection for api_key/auth_token patterns."
    status: completed
  - id: phase4-provider-trait
    content: "Phase 4: Define TranscriptionProvider trait (transcription/provider.rs) with async_trait, TranscriptionResult struct, and ProviderStatus enum. Create provider module structure."
    status: completed
  - id: phase5-provider-implementations
    content: "Phase 5: Implement all four providers (openai.rs, groq.rs, local_whisper.rs, self_hosted.rs) with HTTP clients, local model loading, and configuration management."
    status: completed
  - id: phase6-model-manager
    content: "Phase 6: Implement model download manager (transcription/model_manager.rs) with download, verification, and platform-specific storage paths for Whisper models."
    status: completed
  - id: phase7-transcription-queue
    content: "Phase 7: Implement transcription queue system (transcription/queue.rs) with tokio::sync::mpsc channel, worker thread, and Tauri event emission for status updates."
    status: completed
  - id: phase8-tauri-commands
    content: "Phase 8: Create Tauri commands (commands/audio.rs, commands/transcription.rs) following existing command patterns. Register commands in lib.rs and update commands/mod.rs exports."
    status: completed
  - id: phase9-error-handling
    content: "Phase 9: Add error variants to error.rs, update Cargo.toml with all dependencies, initialize queue in lib.rs, and complete integration testing."
    status: completed
---

# Audio Recording & Transcription Backend Implementation Plan

## Overview

This plan implements the Rust backend for audio recording and transcription capabilities in the Tauri journaling app. The implementation follows existing codebase patterns: repository pattern for database access, Tauri commands for frontend interface, migration-based schema management, and consistent error handling.

## Architecture Overview

```
src/
├── audio/
│   ├── mod.rs              -- Public API & module exports
│   ├── storage.rs          -- Media filesystem operations (save, retrieve, delete)
│   └── recorder.rs         -- Recording metadata handling
├── transcription/
│   ├── mod.rs              -- Public API & orchestration
│   ├── provider.rs         -- Provider trait definition
│   ├── providers/
│   │   ├── mod.rs          -- Provider module exports
│   │   ├── openai.rs       -- OpenAI Whisper implementation
│   │   ├── groq.rs         -- Groq implementation
│   │   ├── local_whisper.rs -- Local Whisper implementation
│   │   └── self_hosted.rs  -- Self-hosted implementation
│   ├── model_manager.rs    -- Model download & verification
│   └── queue.rs            -- Async job processing
├── commands/
│   ├── audio.rs            -- Audio recording commands
│   └── transcription.rs    -- Transcription commands
├── db/
│   ├── repositories/
│   │   ├── media.rs        -- Media repository
│   │   └── transcription.rs -- Transcription repository
│   └── models.rs            -- Add MediaItem, AudioTranscription models
└── settings/
    ├── mod.rs              -- Settings management
    └── encryption.rs       -- Credential encryption/decryption
```

## Database Schema

### Migration: `004_add_audio_transcription_tables.sql`

Creates two new tables and modifies the existing `settings` table:

1. **media_items** - Unified media storage (audio, images, future video)
2. **audio_transcriptions** - Multiple transcription attempts per audio
3. **settings** - Modified to be a flexible key-value store (migrated from singleton)

**Settings Table Migration:**

The existing `settings` table currently has:

- `id TEXT PRIMARY KEY` (singleton with `id = "default"`)
- `timezone TEXT NOT NULL DEFAULT 'UTC'`
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`

**Migration steps:**

1. Create new `settings` table structure as key-value store:

   - `key TEXT PRIMARY KEY` (e.g., "timezone", "transcription.default_provider")
   - `value TEXT NOT NULL` (encrypted if sensitive)
   - `updated_at TEXT NOT NULL`

2. Migrate existing timezone data:

   - Read timezone from old `settings` table (if exists)
   - Insert as `key = "timezone"`, `value = <timezone_value>`

3. Drop old `settings` table
4. Create index on `key` for fast lookups

**Key design decisions:**

- Use `TEXT` for timestamps (RFC3339 strings) matching existing pattern
- Use `INTEGER` for booleans (0/1) matching existing pattern
- Store media files on filesystem, reference via file path in database
- Foreign key constraints with `ON DELETE CASCADE`
- Indexes on frequently queried columns
- `settings.key` is PRIMARY KEY (one value per key)
- `settings.value` stores encrypted values for sensitive keys
- Flexible structure supports future settings (theme, font, etc.) without schema changes

**Media Storage Architecture:**

- **Filesystem storage**: Media files stored in platform-specific directory
  - macOS: `~/Library/Application Support/Aether/media/`
  - Linux: `~/.local/share/aether/media/`
  - Windows: `%APPDATA%/Aether/media/`
- **Database metadata**: `media_items` table stores file path and metadata
- **File naming**: Use UUID-based filenames (e.g., `{media_id}.webm`)
- **Directory structure**: Flat structure (all files in media/ root) or organized by date (media/YYYY/MM/)
- **Initialization**: Ensure media directory exists on app startup
- **Benefits**: Better performance for large files, easier backup/export, no database bloat, handles files of any size

### Models

Add to `db/models.rs`:

- `MediaItem` - Represents audio/image/video with file path and metadata JSON
- `AudioTranscription` - Represents transcription attempt with provider info
- Update `Settings` model to key-value structure:

**MediaItem model structure:**

```rust
pub struct MediaItem {
    pub id: String,
    pub entry_id: String,
    pub media_type: String,        // "audio" | "image" | "video"
    pub file_path: String,          // Relative path from media directory
    pub metadata: serde_json::Value, // JSON: duration, format, size, width, height
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Updated Settings model structure:**

```rust
pub struct Setting {
    pub key: String,           // Primary key (e.g., "timezone", "transcription.default_provider")
    pub value: String,         // Value (encrypted if sensitive)
    pub updated_at: DateTime<Utc>,
}
```

**Media Items Table Schema:**

```sql
CREATE TABLE media_items (
  id TEXT PRIMARY KEY,
  entry_id TEXT NOT NULL,
  media_type TEXT NOT NULL CHECK(media_type IN ('audio', 'image', 'video')),
  file_path TEXT NOT NULL,     -- Relative path from media directory
  metadata TEXT NOT NULL,      -- JSON format
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE
);

CREATE INDEX idx_media_entry_id ON media_items(entry_id);
CREATE INDEX idx_media_type ON media_items(media_type);
```

**Transcription Settings (for this implementation):**

- `transcription.default_provider` → "openai" | "groq" | "local-whisper" | "self-hosted"
- `transcription.auto_transcribe` → "true" | "false"
- `transcription.openai.enabled` → "true" | "false"
- `transcription.openai.api_key` → `<encrypted>`
- `transcription.groq.enabled` → "true" | "false"
- `transcription.groq.api_key` → `<encrypted>`
- `transcription.local_whisper.enabled` → "true" | "false"
- `transcription.local_whisper.model_size` → "tiny" | "base" | "small"
- `transcription.local_whisper.model_path` → "/path/to/models"
- `transcription.local_whisper.downloaded` → "true" | "false"
- `transcription.self_hosted.enabled` → "true" | "false"
- `transcription.self_hosted.endpoint` → "http://localhost:8000"
- `transcription.self_hosted.auth_token` → `<encrypted>`

**Note:** The table structure is flexible and will support future settings like `editor.theme`, `editor.font_size`, `ui.sidebar_collapsed`, etc. without requiring schema changes.

**Sensitive keys (auto-encrypted):**

- Any key containing `api_key`
- Any key containing `auth_token`
- Any key containing `password`
- Any key containing `secret`

## Implementation Phases

### Phase 1: Database Foundation

**Files to create/modify:**

1. **Migration file**: `desktop/src-tauri/migrations/004_add_audio_transcription_tables.sql`

   - Create `media_items` table with file path storage (not BLOB)
   - Create `audio_transcriptions` table
   - Migrate existing `settings` table to key-value structure:
     - Backup existing timezone value (if settings table exists and has data)
     - Drop old `settings` table
     - Create new `settings` table (key-value: `key TEXT PRIMARY KEY`, `value TEXT NOT NULL`, `updated_at TEXT NOT NULL`)
     - Insert timezone as `key = "timezone"` if it existed
     - Create index on `key` for fast lookups
   - Add indexes for performance on new tables

2. **Schema creation**: `desktop/src-tauri/src/db/schema.rs`

   - Update `create_schema()` function to create new `settings` table structure (key-value)
   - Remove old singleton `settings` table creation
   - This ensures new installations get the correct structure

3. **Models**: `desktop/src-tauri/src/db/models.rs`

   - Add `MediaItem` struct with serde annotations
   - Add `AudioTranscription` struct
   - Update `Settings` struct to key-value structure (replace existing singleton model)
   - Use `chrono::DateTime<Utc>` for timestamps (matches existing pattern)

4. **Repositories**: 

   - `desktop/src-tauri/src/db/repositories/media.rs` - Media CRUD operations
   - `desktop/src-tauri/src/db/repositories/transcription.rs` - Transcription operations
   - `desktop/src-tauri/src/db/repositories/settings.rs` - Settings operations (key-value store)
   - Update `desktop/src-tauri/src/db/repositories/mod.rs` to export new repositories

**Repository pattern** (following `EntryRepository`):

- `MediaRepository::new(database: Arc<Database>)`
- Methods: `create()`, `find_by_id()`, `find_by_entry_id()`, `get_file_path()`, `delete()` (also deletes file from filesystem)
- `TranscriptionRepository::new(database: Arc<Database>)`
- Methods: `create()`, `find_by_media_id()`, `set_active()`, `update_status()`, `find_by_id()`
- `SettingsRepository::new(database: Arc<Database>)`
- Methods: `get(key: &str)`, `set(key: &str, value: &str)`, `delete(key: &str)`, `get_all()`, `get_by_prefix(prefix: &str)`

### Phase 2: Audio Storage Module

**Files to create:**

1. **`desktop/src-tauri/src/audio/mod.rs`**

   - Module exports
   - Public API functions

2. **`desktop/src-tauri/src/audio/storage.rs`**

   - `save_audio_file()` - Save audio file to filesystem and metadata to database
   - `get_audio_file_path()` - Get file path for audio
   - `read_audio_file()` - Read audio file bytes from filesystem
   - `delete_audio()` - Delete media item, transcriptions (CASCADE), and file from filesystem
   - Platform-specific media directory management
   - Uses `MediaRepository` for database operations
   - Uses `std::fs` for filesystem operations

3. **`desktop/src-tauri/src/audio/recorder.rs`**

   - Metadata extraction from audio format
   - Duration calculation helpers
   - Format detection utilities
   - File extension determination from format

**Design notes:**

- Audio data passed as `Vec<u8>` from frontend
- File saved to filesystem with UUID-based filename (e.g., `{media_id}.webm`)
- File path stored in database (relative to media directory)
- Metadata stored as JSON string in database
- Use existing `generate_id()` utility for IDs
- Ensure media directory exists before saving
- Handle filesystem errors gracefully

### Phase 3: Settings Management with Encryption

**Files to create:**

1. **`desktop/src-tauri/src/settings/mod.rs`**

   - `get_setting(key: &str)` - Retrieve setting (auto-decrypt)
   - `set_setting(key: &str, value: &str)` - Store setting (auto-encrypt)
   - `delete_setting(key: &str)` - Remove setting
   - Pattern matching for sensitive keys (`*api_key*`, `*auth_token*`)

2. **`desktop/src-tauri/src/settings/encryption.rs`**

   - AES-256-GCM encryption/decryption
   - OS keychain integration (use `tauri-plugin-keyring` or platform-specific APIs)
   - Key derivation from OS keychain
   - Automatic encryption detection

**Dependencies to add:**

- `aes-gcm = "0.10"`
- `base64 = "0.21"`
- `tauri-plugin-keyring = "2"` (or platform-specific keychain crates)

**Settings key structure (examples for transcription):**

- `timezone` → "UTC" (migrated from old settings table)
- `transcription.default_provider` → "openai"
- `transcription.auto_transcribe` → "true"
- `transcription.openai.api_key` → `<encrypted>`
- `transcription.local_whisper.model_path` → "/path/to/models"

**Note:** The key-value structure is flexible and will support future settings without schema changes.

### Phase 4: Transcription Provider Trait

**Files to create:**

1. **`desktop/src-tauri/src/transcription/mod.rs`**

   - Module exports
   - Provider manager/registry
   - Public API functions

2. **`desktop/src-tauri/src/transcription/provider.rs`**

   - `TranscriptionProvider` trait definition
   - `TranscriptionResult` struct
   - `ProviderStatus` enum
   - Common provider utilities

**Trait definition:**

```rust
#[async_trait]
pub trait TranscriptionProvider: Send + Sync {
    fn name(&self) -> &str;
    fn requires_api_key(&self) -> bool;
    fn requires_download(&self) -> bool;
    
    async fn initialize(&mut self) -> Result<(), String>;
    async fn transcribe(&self, audio_data: &[u8], format: &str) 
        -> Result<TranscriptionResult, String>;
    async fn get_status(&self) -> ProviderStatus;
    async fn validate_config(&self) -> Result<(), String>;
}
```

**Dependencies to add:**

- `async-trait = "0.1"`

### Phase 5: Provider Implementations

**Files to create:**

1. **`desktop/src-tauri/src/transcription/providers/mod.rs`**

   - Provider module exports

2. **`desktop/src-tauri/src/transcription/providers/openai.rs`**

   - OpenAI Whisper API client
   - Multipart form upload
   - API key from settings (decrypted)
   - Error handling for rate limits/timeouts

3. **`desktop/src-tauri/src/transcription/providers/groq.rs`**

   - Groq API client (similar to OpenAI)
   - Compatible API interface

4. **`desktop/src-tauri/src/transcription/providers/local_whisper.rs`**

   - Local Whisper model loading
   - CPU/GPU detection
   - Audio processing with `whisper-rs` or `candle`
   - Wrapped in `tokio::task::spawn_blocking` for CPU work

5. **`desktop/src-tauri/src/transcription/providers/self_hosted.rs`**

   - HTTP client for custom endpoint
   - Health check via `/health`
   - Optional authentication token

**Dependencies to add:**

- `reqwest = { version = "0.11", features = ["json", "multipart"] }`
- `whisper-rs = "0.10"` (or `candle` alternative)
- `hound = "3.5"` (for audio format handling)

**Provider initialization:**

- Load configuration from `settings` table (key-value)
- Decrypt API keys/tokens as needed
- Validate configuration on startup
- Cache provider instances (avoid re-initialization)

### Phase 6: Model Download Manager

**Files to create:**

1. **`desktop/src-tauri/src/transcription/model_manager.rs`**

   - `list_available_models()` - Model catalog
   - `download_model(size: &str)` - Download with progress
   - `verify_model(size: &str)` - Checksum verification
   - `delete_model(size: &str)` - Remove model files
   - Platform-specific storage paths

**Model storage:**

- macOS: `~/Library/Application Support/Aether/models/`
- Linux: `~/.local/share/aether/models/`
- Windows: `%APPDATA%/Aether/models/`

**Download process:**

- Use `reqwest` for HTTP download
- Progress tracking via Tauri events
- SHA-256 checksum verification
- Extract/verify model files
- Update settings to mark as downloaded

**Dependencies to add:**

- `sha2 = "0.10"` (for checksums)
- `flate2 = "1.0"` (for extraction if needed)

### Phase 7: Transcription Queue System

**Files to create:**

1. **`desktop/src-tauri/src/transcription/queue.rs`**

   - In-memory job queue using `tokio::sync::mpsc`
   - Single worker thread for sequential processing
   - Tauri event emission for status updates
   - Database updates on completion

**Queue architecture:**

- `TranscriptionJob` struct (media_id, provider_name, transcription_id)
- `JobQueue` struct with channel
- Worker loop processes jobs sequentially
- Emit events: `transcription:status_changed`
- Update database status: pending → processing → complete/failed

**Event payload:**

```rust
{
    media_id: String,
    status: "pending" | "processing" | "complete" | "failed",
    text?: String,
    error?: String,
    progress?: f32
}
```

**Integration:**

- Initialize queue in `lib.rs` during app startup
- Store queue handle in app state (if needed)
- Worker runs in background tokio task

### Phase 8: Tauri Commands

**Files to create:**

1. **`desktop/src-tauri/src/commands/audio.rs`**

   - `save_audio_recording()` - Save audio file to filesystem and metadata to database, optionally queue transcription
   - `get_audio_data()` - Retrieve audio file bytes from filesystem
   - `delete_audio()` - Delete media item, transcriptions, and file from filesystem

2. **`desktop/src-tauri/src/commands/transcription.rs`**

   - `start_transcription()` - Manually trigger transcription
   - `get_transcriptions()` - Get all transcriptions for audio
   - `set_active_transcription()` - Mark transcription as active
   - `list_providers()` - Provider status info
   - `validate_provider()` - Test provider config
   - `list_available_models()` - Whisper model catalog
   - `download_model()` - Download with progress events
   - `verify_model()` - Check model integrity
   - `delete_model()` - Remove model files
   - `get_setting()` - Retrieve setting value
   - `set_setting()` - Store setting value

**Command pattern** (following existing commands):

- Use `#[tauri::command]` macro
- Use `State<'_, DbState>` for database access
- Use `#[utoipa::path]` for OpenAPI annotations
- Return `Result<T>` for error handling
- Use repository pattern for database operations

**Update `desktop/src-tauri/src/commands/mod.rs`:**

- Export `audio` and `transcription` modules

**Update `desktop/src-tauri/src/lib.rs`:**

- Register new commands in `invoke_handler![]` macro

### Phase 9: Error Handling & Integration

**Files to modify:**

1. **`desktop/src-tauri/src/error.rs`**

   - Add error variants:
     - `TranscriptionError(String)` - Provider-specific errors
     - `ModelError(String)` - Model download/verification errors
     - `EncryptionError(String)` - Encryption/decryption failures
     - `ProviderNotConfigured(String)` - Missing configuration

2. **`desktop/src-tauri/Cargo.toml`**

   - Add all required dependencies
   - Ensure feature flags are correct

3. **`desktop/src-tauri/src/lib.rs`**

   - Initialize media directory on startup (ensure it exists)
   - Initialize transcription queue on startup
   - Register all new commands
   - Set up event emission (if needed)

## Key Implementation Details

### Database Operations

**Media storage:**

- Store audio files in platform-specific media directory
- Store file path in `media_items.file_path` (relative path)
- Metadata JSON includes: duration, format, size, width, height
- Use transactions for atomic operations (file write + DB insert)
- Foreign key ensures cascade delete of transcriptions
- On delete: Remove file from filesystem and database record

**Transcription storage:**

- Multiple transcriptions per media_id
- `is_active` flag marks selected transcription
- Status tracking: pending → processing → complete/failed
- Provider config stored as JSON for debugging

### Settings Encryption

**Encryption flow:**

1. Check if key matches sensitive pattern (`*api_key*`, `*auth_token*`)
2. If sensitive: encrypt value with AES-256-GCM
3. Store encrypted value in database
4. Encryption key stored in OS keychain
5. On read: detect encrypted value, decrypt automatically

**Key derivation:**

- Generate or retrieve master key from OS keychain
- Use key for all encryption operations
- Key never stored in database

### Provider Management

**Provider registry:**

- Static list of available providers
- Lazy initialization on first use
- Configuration loaded from `user_settings`
- Status cached and updated on validation

**Provider selection:**

- Default provider from settings
- Fallback to first available provider
- User can override per transcription

### Audio Format Handling

**Supported formats:**

- WebM (Opus) - from browser MediaRecorder
- MP3, WAV, M4A - for compatibility
- Format detection via file headers
- Conversion may be needed for some providers

**Format conversion:**

- Use `hound` crate for WAV handling
- Consider `ffmpeg` bindings if needed (optional)
- Most providers accept multiple formats

## Testing Strategy

### Unit Tests

**Repositories:**

- Media CRUD operations
- Transcription operations
- Settings key-value operations
- Settings encryption/decryption

**Providers:**

- Mock HTTP responses for API providers
- Test configuration validation
- Test error handling

### Integration Tests

**End-to-end flows:**

- Record → Save → Transcribe → Retrieve
- Multiple provider switching
- Model download and verification
- Settings encryption round-trip

### Edge Cases

- Large audio files (filesystem handles efficiently)
- Missing media files (file deleted outside app)
- Disk space exhaustion
- Network failures during transcription
- Invalid API keys
- Corrupted model files
- Concurrent transcription requests
- Filesystem permission errors

## Dependencies Summary

**New dependencies for `Cargo.toml`:**

```toml
# Audio processing
hound = "3.5"

# HTTP clients
reqwest = { version = "0.11", features = ["json", "multipart"] }

# Encryption
aes-gcm = "0.10"
base64 = "0.21"

# Async traits
async-trait = "0.1"

# Local Whisper (choose one)
whisper-rs = "0.10"  # Or candle-based alternative

# Model verification
sha2 = "0.10"

# Keychain (platform-specific or tauri plugin)
# tauri-plugin-keyring = "2"  # If available
# Or use platform-specific crates
```

## Migration Path

1. **Phase 1-2**: Database and storage (migrates existing settings table to key-value)
2. **Phase 3**: Settings system (key-value store with encryption)
3. **Phase 4-5**: Provider infrastructure (no frontend impact yet)
4. **Phase 6**: Model management (optional feature)
5. **Phase 7**: Queue system (background processing)
6. **Phase 8**: Commands (frontend integration ready)

Each phase can be tested independently before moving to the next.

## Performance Considerations

**Audio storage:**

- Filesystem storage handles files of any size efficiently
- Typical 5-minute recording: ~5MB (WebM/Opus)
- Database only stores metadata (small, fast queries)
- File I/O is efficient for large files
- No database bloat from large media files
- Easier backup/export (can copy media directory separately)

**Transcription queue:**

- Sequential processing prevents resource exhaustion
- Background processing doesn't block UI
- Status updates via events keep UI responsive

**Local Whisper:**

- CPU transcription: 2-5x real-time (10-25 min for 5 min audio)
- GPU transcription: 0.5-1x real-time (2.5-5 min for 5 min audio)
- Model loading: ~1-2 seconds (keep in memory if possible)

## Security Considerations

**API key storage:**

- Always encrypted at rest
- Never logged or exposed in errors
- Secure keychain access
- Automatic encryption detection

**Audio data:**

- Stored locally only
- No transmission to unintended services
- User controls provider selection
- Self-hosted option for full control