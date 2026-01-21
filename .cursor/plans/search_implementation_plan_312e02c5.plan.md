---
name: Search Implementation Plan
overview: Build comprehensive backend search across all resources using LibSQL's native FTS5 trigram for fuzzy search and local embedding models (all-MiniLM-L6-v2) for semantic similarity search, with hybrid search combining both. Includes full local model implementation with download, loading, and inference.
todos:
  - id: phase1-migration
    content: Create migration 005_add_search_indexes.sql with FTS5 virtual tables and triggers for all resources
    status: pending
  - id: phase1-repository
    content: Implement SearchRepository in db/repositories/search.rs with fuzzy search methods
    status: pending
  - id: phase1-handler
    content: Create search handler in handlers/search.rs with GET /v1/search endpoint
    status: pending
  - id: phase1-command
    content: Create Tauri command in commands/search.rs
    status: pending
  - id: phase1-tests
    content: Add unit tests for SearchRepository and integration tests for search endpoints
    status: pending
  - id: phase2-embeddings-migration
    content: Create migration 006_add_vector_embeddings.sql with F32_BLOB columns and vector indexes
    status: pending
  - id: phase2-model-manager
    content: Implement complete model manager in utils/embeddings/model_manager.rs with download, verification, and storage following transcription pattern
    status: pending
  - id: phase2-model-download
    content: Implement model download from HuggingFace (safetensors, tokenizer, config) with progress tracking and checksum verification
    status: pending
  - id: phase2-embedding-generator
    content: Replace placeholder in generator.rs with actual model loading and inference using glowrs or candle-transformers
    status: pending
  - id: phase2-model-loading
    content: Implement model loading from safetensors format, tokenizer loading, and device selection (CPU/Metal/CUDA)
    status: pending
  - id: phase2-inference
    content: Implement tokenization, model inference, mean-pooling, and L2 normalization to generate 384-dim embeddings
    status: pending
  - id: phase2-auto-generation
    content: Add embedding generation hooks to all repository create/update methods (Entry, Task, SubTask, Goal, Tag)
    status: pending
  - id: phase2-backfill
    content: Create background job and Tauri commands to generate embeddings for existing data with progress tracking
    status: pending
  - id: phase2-similar-search
    content: Add similar search method to SearchRepository using vector_top_k
    status: pending
  - id: phase2-embedding-tests
    content: Add tests for embedding generation, model loading, similar search, and quality validation
    status: pending
  - id: phase3-hybrid
    content: Implement hybrid search mode combining FTS5 and vector search with weighted ranking
    status: pending
  - id: phase3-hybrid-tests
    content: Add integration tests for hybrid search ranking and result merging
    status: pending
---

# Search Implementation Plan

## Architecture Overview

Using LibSQL's native capabilities:

- **FTS5 with trigram tokenizer**: Fuzzy/substring search (typo-tolerant)
- **Local embedding models**: Semantic similarity search using all-MiniLM-L6-v2 (384 dims)
- **Hybrid search**: Combine both for comprehensive results

## Current Resources to Search

- **Entries**: `document` field (rich text/JSON)
- **Tasks**: `title`, `description` fields
- **Subtasks**: `title` field
- **Goals**: `name`, `description` fields
- **Tags**: `name` field

## Implementation Phases

### Phase 1: FTS5 Fuzzy Search (Foundation)

#### Database Migration: `005_add_search_indexes.sql`

Create FTS5 virtual tables with trigram tokenizer for all resources with triggers to keep indexes in sync and backfill existing data.

#### Backend Implementation

**Files created:**

- [`desktop/src-tauri/src/db/repositories/search.rs`](desktop/src-tauri/src/db/repositories/search.rs) - SearchRepository with fuzzy search
- [`desktop/src-tauri/src/handlers/search.rs`](desktop/src-tauri/src/handlers/search.rs) - GET /v1/search endpoint
- [`desktop/src-tauri/src/commands/search.rs`](desktop/src-tauri/src/commands/search.rs) - Tauri command wrapper

**Status**: Completed

### Phase 2: Vector Embeddings with Local Model

#### Database Migration: `006_add_vector_embeddings.sql`

Add F32_BLOB(384) columns and vector indexes using `libsql_vector_idx` for all searchable resources.

**Status**: Completed

#### Embedding Model Manager

**File**: [`desktop/src-tauri/src/utils/embeddings/model_manager.rs`](desktop/src-tauri/src/utils/embeddings/model_manager.rs)

Following the pattern from [`desktop/src-tauri/src/transcription/model_manager.rs`](desktop/src-tauri/src/transcription/model_manager.rs):

**Functions to implement:**

- `list_available_models() -> Vec<ModelInfo>` - Model catalog
  - `all-MiniLM-L6-v2` (384 dims, ~80MB, recommended)
  - Optional: `all-mpnet-base-v2` (768 dims, larger, better quality)
- `download_model(name: &str, progress_callback: Option<Box<dyn Fn(f32) + Send + Sync>>) -> Result<PathBuf>`
  - Download from HuggingFace (safetensors format)
  - Progress tracking via callback
  - Store in `models/embeddings/` directory
  - Update settings to mark as downloaded
- `verify_model(name: &str) -> Result<bool>` - SHA-256 checksum verification
- `get_model_path(name: &str) -> PathBuf` - Platform-specific storage path
- `is_model_downloaded(name: &str) -> bool` - Check if model exists
- `delete_model(name: &str) -> Result<()>` - Remove model files

**Model storage paths** (matching transcription pattern):

- macOS: `~/Library/Application Support/Aether/models/embeddings/`
- Linux: `~/.local/share/aether/models/embeddings/`
- Windows: `%APPDATA%/Aether/models/embeddings/`

**Model download URLs:**

- `all-MiniLM-L6-v2`: HuggingFace safetensors format
- Download both model weights and tokenizer files

#### Embedding Generator with Local Model

**File**: [`desktop/src-tauri/src/utils/embeddings/generator.rs`](desktop/src-tauri/src/utils/embeddings/generator.rs)

**Current state**: Placeholder implementation exists. Replace with actual model loading and inference.

**Implementation approach** (choose one):

**Option 1: Using `glowrs` (Recommended - simpler)**

- High-level wrapper around Candle
- Handles tokenization, inference, pooling, normalization automatically
- Easier to implement and maintain

**Option 2: Using `candle-transformers` directly (More control)**

- Lower-level, more control over inference
- Requires manual tokenization, pooling, normalization
- Better for custom requirements

**Dependencies to add to [`desktop/src-tauri/Cargo.toml`](desktop/src-tauri/Cargo.toml):**

```toml
# Option 1: Using glowrs (recommended)
glowrs = "0.1"  # Check latest version

# Option 2: Using candle-transformers directly
# candle-core = { version = "0.3", features = ["metal", "cuda"] }
# candle-transformers = "0.3"
# candle-nn = "0.3"

# Required for both options
tokenizers = "0.15"  # For tokenization
safetensors = "0.4"  # For loading model weights
```

**Implementation details for `generator.rs`:**

1. **Model structure**:
```rust
struct EmbeddingModel {
    encoder: SentenceTransformer, // if using glowrs
    // OR
    model: BertModel,              // if using candle-transformers directly
    tokenizer: Tokenizer,
    device: Device,
    loaded: bool,
}
```

2. **Model loading** (`init_model()` function):

   - Check if model is downloaded via `model_manager::is_model_downloaded()`
   - If not downloaded, trigger download (or return error)
   - Load model from safetensors format
   - Load tokenizer from HuggingFace tokenizer files
   - Cache model in memory using `OnceCell` or `Arc<Mutex<>>`
   - Handle device selection (CPU, Metal for macOS, CUDA if available)

3. **Embedding generation** (`generate_embedding()` function):

   - Tokenize input text (max 256 tokens, truncate if longer)
   - Run model inference (forward pass)
   - Apply mean-pooling over token embeddings (weighted by attention mask)
   - L2 normalize the resulting vector
   - Return 384-dimensional `Vec<f32>`
   - Wrap CPU-intensive work in `tokio::task::spawn_blocking`

4. **Error handling**:

   - Model not downloaded → return clear error message
   - Model loading failure → log and return error
   - Inference failure → log and return error
   - Empty text → return BadRequest error

**Key implementation notes:**

- Pooling strategy must match Python's sentence-transformers (mean-pooling over real tokens, excluding padding)
- Normalization is critical (L2 normalize to unit vector)
- Model should be loaded once and cached (lazy initialization)
- Use `tokio::task::spawn_blocking` for model inference (CPU-intensive)
- Handle device selection gracefully (fallback to CPU if Metal/CUDA unavailable)

#### Model Download Implementation

**Extend `model_manager.rs`:**

1. **Model catalog**:
```rust
pub fn list_available_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            name: "all-MiniLM-L6-v2".to_string(),
            size: "small".to_string(),
            dimensions: 384,
            file_size: 80_000_000, // ~80MB
            download_url: "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/model.safetensors".to_string(),
            tokenizer_url: "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json".to_string(),
            config_url: "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/config.json".to_string(),
            checksum: Some("...".to_string()), // SHA-256
            downloaded: false,
        },
    ]
}
```

2. **Download function**:

   - Download model.safetensors
   - Download tokenizer.json
   - Download config.json (optional, for verification)
   - Verify checksums
   - Store in `models/embeddings/all-MiniLM-L6-v2/` directory
   - Update settings: `embedding.model.downloaded_all-MiniLM-L6-v2 = "true"`

3. **Verification**:

   - Check file existence
   - Verify file sizes match expected
   - Optional: SHA-256 checksum verification

#### Automatic Embedding Generation

**Hooks to add in repositories:**

When resources are created/updated, automatically generate embeddings:

1. **EntryRepository** (`create()`, `update()` methods):

   - After successful DB operation, spawn async task to generate embedding
   - Extract text from `document` field (may need to parse JSON/rich text)
   - Call `utils::embeddings::generate_embedding()`
   - Update `embedding` column in database
   - Handle errors gracefully (log but don't fail the operation)

2. **TaskRepository** (`create()`, `update()` methods):

   - Combine `title` and `description` for embedding
   - Generate embedding and store

3. **Similar for SubTask, Goal, Tag repositories**

**Implementation pattern:**

```rust
// In repository create/update methods, after DB operation:
let text_to_embed = format!("{} {}", title, description.unwrap_or_default());
let embedding_task = {
    let db_clone = Arc::clone(&self.database);
    let id_clone = id.clone();
    tokio::spawn(async move {
        match crate::utils::embeddings::generate_embedding(&text_to_embed).await {
            Ok(embedding) => {
                // Convert Vec<f32> to F32_BLOB format for LibSQL
                // Update embedding column
            }
            Err(e) => {
                tracing::warn!("Failed to generate embedding for {}: {}", id_clone, e);
            }
        }
    });
};
// Don't await - let it run in background
```

#### Background Job for Existing Data

**New file**: [`desktop/src-tauri/src/commands/embeddings.rs`](desktop/src-tauri/src/commands/embeddings.rs)

**Tauri commands:**

- `generate_embeddings_for_all()` - Batch process all resources
- `generate_embedding_for_resource(resource_type, resource_id)` - Single resource
- `get_embedding_generation_status()` - Progress tracking

**Implementation:**

- Query all resources without embeddings
- Process in batches (e.g., 10 at a time)
- Generate embeddings and update database
- Emit Tauri events for progress updates
- Handle errors gracefully (continue on failure, log errors)

#### Similar Search Implementation

**Status**: Completed in SearchRepository

Uses `vector_top_k()` function to query vector indexes and find similar resources based on cosine distance.

#### Testing

**Unit tests**: [`desktop/src-tauri/src/utils/embeddings/`](desktop/src-tauri/src/utils/embeddings/)

- Test model download and verification
- Test embedding generation with sample text
- Test embedding vector dimensions (384)
- Test model loading and caching
- Test tokenization and pooling
- Mock model files for testing

**Integration tests**: [`desktop/src-tauri/tests/integration_test.rs`](desktop/src-tauri/tests/integration_test.rs)

- Test embedding generation with real model (if available)
- Test similar search with real embeddings
- Test vector index queries
- Test embedding generation on resource create/update
- Test background embedding generation job

### Phase 3: Hybrid Search

**Status**: Completed

Enhanced search endpoint with `mode` parameter supporting `fuzzy`, `similar`, and `hybrid` modes. Hybrid mode combines FTS5 and vector search with weighted ranking (60% fuzzy + 40% semantic).

## Key Technical Details

### Local Embedding Model

**Model**: `sentence-transformers/all-MiniLM-L6-v2`

- 384-dimensional embeddings
- ~80MB model size
- Fast inference (CPU-friendly)
- Good quality for semantic search
- Max input: 256 tokens (truncate if longer)

**Inference pipeline:**

1. Tokenize text using model's tokenizer
2. Run BERT encoder forward pass
3. Mean-pool token embeddings (weighted by attention mask)
4. L2 normalize to unit vector
5. Return 384-dimensional Vec<f32>

**Model format**: Safetensors (preferred) or PyTorch

**Device support**: CPU (default), Metal (macOS), CUDA (if available)

### LibSQL Vector Support

- Use `F32_BLOB(384)` for 384-dimensional embeddings
- Vector index created with `libsql_vector_idx(embedding, 'metric=cosine')`
- Query with `vector_top_k('index_name', query_vector, k)` table-valued function
- Vector functions: `vector32()`, `vector_distance_cos()`, `vector_extract()`

### FTS5 Trigram

- Enables substring matching and typo tolerance
- Use `MATCH` operator for queries
- Triggers keep indexes in sync automatically
- Backfill existing data on migration

### Performance Considerations

- FTS5 indexes: ~30-50% storage overhead
- Vector embeddings: 384 dims × 4 bytes = 1.5KB per resource
- Model inference: ~10-50ms per text on CPU (depends on length)
- Use pagination (limit/offset) for large result sets
- Vector indexes use DiskANN (approximate nearest neighbor) for speed
- Cache model in memory to avoid reloading

### Extensibility

To add new resource types:

1. Create FTS5 virtual table with triggers
2. Add embedding column (F32_BLOB(384))
3. Create vector index
4. Add to SearchRepository enum
5. Update search queries
6. Add embedding generation hook in repository

## Migration Strategy

1. **Phase 1**: Deploy FTS5 indexes and fuzzy search (works immediately) ✅
2. **Phase 2**: Add embedding columns (nullable initially) ✅
3. **Phase 2**: Implement model manager and download functionality
4. **Phase 2**: Implement embedding generator with local model
5. **Phase 2**: Add automatic embedding generation on create/update
6. **Phase 2**: Background job generates embeddings for existing data
7. **Phase 3**: Enable hybrid search mode ✅

## Testing Considerations

- Test model download from HuggingFace
- Test model loading and caching
- Test embedding generation with various text lengths
- Test embedding quality (compare with Python sentence-transformers)
- Test similar search with real embeddings
- Test hybrid ranking quality
- Performance test with large datasets
- Test offline functionality (local-first)
- Test error handling (model not downloaded, inference failures)

## Dependencies Summary

**New dependencies for `Cargo.toml`:**

```toml
# Embedding model (choose one approach)
glowrs = "0.1"  # Recommended: high-level wrapper
# OR
# candle-core = { version = "0.3", features = ["metal", "cuda"] }
# candle-transformers = "0.3"
# candle-nn = "0.3"

# Required
tokenizers = "0.15"      # Tokenization
safetensors = "0.4"      # Model weight loading
sha2 = "0.10"            # Checksum verification (already added)
reqwest = "0.11"         # HTTP download (already added)
```

## Implementation Checklist

### Model Manager

- [ ] Implement `list_available_models()` with all-MiniLM-L6-v2 info
- [ ] Implement `download_model()` with progress tracking
- [ ] Implement `verify_model()` with checksum verification
- [ ] Implement `get_model_path()` for platform-specific paths
- [ ] Implement `is_model_downloaded()` check
- [ ] Implement `delete_model()` cleanup
- [ ] Add Tauri commands for model management

### Embedding Generator

- [ ] Choose implementation approach (glowrs vs candle-transformers)
- [ ] Implement model loading from safetensors
- [ ] Implement tokenizer loading
- [ ] Implement tokenization (max 256 tokens)
- [ ] Implement model inference (forward pass)
- [ ] Implement mean-pooling (weighted by attention mask)
- [ ] Implement L2 normalization
- [ ] Add device selection (CPU/Metal/CUDA)
- [ ] Add error handling for all failure cases
- [ ] Test embedding quality against reference implementation

### Automatic Generation

- [ ] Add embedding hooks to EntryRepository
- [ ] Add embedding hooks to TaskRepository
- [ ] Add embedding hooks to SubTaskRepository
- [ ] Add embedding hooks to GoalRepository
- [ ] Add embedding hooks to TagRepository
- [ ] Extract text from rich text/JSON formats
- [ ] Handle async generation without blocking

### Background Job

- [ ] Implement batch processing for existing data
- [ ] Add progress tracking
- [ ] Add Tauri events for status updates
- [ ] Handle errors gracefully
- [ ] Add resume capability (skip already-embedded resources)

### Testing

- [ ] Unit tests for model manager
- [ ] Unit tests for embedding generator
- [ ] Integration tests for end-to-end flow
- [ ] Performance benchmarks
- [ ] Quality validation tests