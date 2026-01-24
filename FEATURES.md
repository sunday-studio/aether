# Aether Feature List

## 1. Content Management

### 1.1 Journal Entries
**Operations:** Create, Read, Update, Delete  
**Features:**
- Rich text editor (Lexical)
- Timeline view
- Grid view
- Tagging
- Resource linking
- Activity tracking

### 1.2 Tasks
**Operations:** Create, Read, Update, Delete  
**Features:**
- Inbox view
- Overdue view
- Subtasks with reordering
- Completion tracking
- Due dates
- Descriptions
- Goal assignment
- Tagging
- Resource linking
- Trash/restore
- Activity tracking

### 1.3 Goals
**Operations:** Create, Read, Update, Delete  
**Features:**
- Recurrence types: daily, weekly, monthly, yearly, custom
- Goal instances
- Timezone-aware periods
- Task assignment
- Tagging
- Resource linking
- Activity tracking

### 1.4 Canvas
**Operations:** Create, Read, Update, Delete  
**Features:**
- JSON Canvas format
- Pan and zoom
- Node types: text, file, link
- Edge connections
- Drag and drop
- Multi-select
- Undo/redo
- Import/export
- Resource linking

### 1.5 Bookmarks
**Operations:** Create, Read, Update, Delete  
**Features:**
- Automatic metadata extraction
- Open Graph support
- Twitter Card support
- External API fallback
- Tagging
- Resource linking
- Grid/list views
- Filtering (tags, content type, date)
- Chrome extension (planned)

## 2. Search & Discovery

### 2.1 Search
**Operations:** Query  
**Modes:**
- Fuzzy search (FTS5 trigram)
- Semantic search (embeddings)
- Hybrid search (combined)
**Filters:**
- Resource type
- Tags
- Pagination

### 2.2 Resource Linking
**Operations:** Create, Read, Delete  
**Features:**
- Bidirectional links (`[[]]` syntax)
- Autocomplete search
- Backlink display
- Knowledge graph visualization
- Resource navigation

### 2.3 Graph View
**Operations:** View, Navigate  
**Features:**
- Force-directed layout
- Node visualization by type
- Edge visualization
- Type filtering
- Resource search and focus
- Navigation to resources

## 3. Media & Transcription

### 3.1 Audio Recording
**Operations:** Create, Read, Delete  
**Features:**
- Audio file recording
- Entry attachment
- Playback

### 3.2 Transcription
**Operations:** Create, Read, Update, Delete  
**Providers:**
- OpenAI Whisper API
- Groq API
- Local Whisper models
- Self-hosted endpoints
**Features:**
- Automatic transcription (optional)
- Manual trigger
- Multiple transcriptions per audio
- Active transcription selection
- Status tracking
- Model management

### 3.3 Media Management
**Operations:** Create, Read, Delete  
**Types:** Audio, Image, Video  
**Features:**
- Entity-agnostic storage
- Blurhash generation (images)
- Metadata storage
- Platform-specific directories

## 4. Organization

### 4.1 Tags
**Operations:** Create, Read, Update, Delete, Bulk Create  
**Features:**
- Tag entries, tasks, goals, bookmarks
- Search filtering

### 4.2 Activity Tracking
**Operations:** Read  
**Features:**
- Action logging (create, update, delete, complete)
- Date-based counts
- Entity type breakdown
- Action type breakdown
- Heatmap visualization
- Audit log

### 4.3 Trash
**Operations:** Read, Restore  
**Features:**
- Soft delete (tasks)
- Restore functionality
- Trash view

## 5. Intelligence

### 5.1 Embeddings
**Operations:** Generate, Query  
**Features:**
- Local model (all-MiniLM-L6-v2)
- 384-dimensional vectors
- Automatic generation
- Background processing
- Model download (HuggingFace)
- Model verification
- Vector similarity search

## 6. Configuration

### 7.1 Settings
**Operations:** Read, Write  
**Storage:** Key-value store  
**Features:**
- Encrypted sensitive keys
- Timezone preference
- Theme settings
- Transcription provider settings
- Embedding model settings

### 7.2 Theme System
**Operations:** Configure, Switch  
**Modes:** Light, Dark, System  
**Variants:** Light, Dark, Warm Yellow, Darker  
**Features:**
- Tailwind CSS v4 configuration
- System preference detection
- Theme switching

## 8. Technical Infrastructure

### 8.1 Database
**Technology:** LibSQL (local-only)  
**Features:**
- SQLite compatibility
- FTS5 full-text search
- Vector embeddings (cosine similarity)
- Migration system
- Transaction support
- Local database storage

### 8.2 Backend
**Technology:** Rust, Tauri  
**Features:**
- Repository pattern
- RESTful API (OpenAPI)
- TypeScript SDK generation
- Error handling
- Logging (tracing)

### 8.3 Frontend
**Technology:** React, TypeScript  
**Features:**
- TanStack Query
- Zustand state management
- React Router
- Lexical editor
- React Konva (canvas)
- Command palette
- Keyboard shortcuts

### 8.4 Platform Support
**Platforms:** macOS, Linux, Windows  
**Features:**
- Desktop application (Tauri)
- Platform-specific media directories
- Platform-specific model storage
