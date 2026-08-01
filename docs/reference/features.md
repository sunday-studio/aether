# Aether V1 Feature List

This is the v1 product inventory. It should not list deferred implementation as shipped behavior.

## 1. Content Management

### 1.1 Journal Entries

**Operations:** Create, read, update, delete, restore.

**Features:**

- Rich text editor.
- Timeline view.
- Grid view.
- Tagging.
- Image and video media blobs.
- Activity tracking.
- Trash/restore.
- Local search indexing.

### 1.2 Tasks

**Operations:** Create, read, update, delete, restore.

**Features:**

- Inbox view.
- Overdue view.
- Subtasks with reordering.
- Completion tracking.
- Due dates.
- Descriptions.
- Goal assignment.
- Tagging.
- Trash/restore.
- Activity tracking.
- Local search indexing.

### 1.3 Goals

**Operations:** Create, read, update, delete.

**Features:**

- Recurrence types.
- Goal instances.
- Timezone-aware periods.
- Task assignment.
- Tagging.
- Activity tracking.

## 2. Search

### 2.1 Journal And Task Search

**Operations:** Query, reindex, inspect status.

**Modes:**

- Keyword search.
- Semantic search when a local embedding model is installed.
- Hybrid search.

**Filters:**

- Resource type: `entry` or `task`.
- Tags.
- Date range.
- Pagination.

### 2.2 Local Search Model Setup

**Operations:** List, download, verify, delete, and configure the local embedding model used by journal/task search.

## 3. Media

### 3.1 Image And Video Media Blobs

**Operations:** Store, read, sync, and delete media metadata/files.

**Types:** Image and video.

**Features:**

- File-backed local storage.
- Blurhash generation for images.
- Metadata storage.
- Encrypted blob sync compatibility.

## 4. Organization

### 4.1 Tags

**Operations:** Create, read, update, delete, bulk create.

**Features:**

- Tag journal entries, tasks, and goals.
- Filter journal/task search results by tag.

### 4.2 Activity Tracking

**Operations:** Read.

**Features:**

- Action logging.
- Date-based counts.
- Entity type breakdown.
- Action type breakdown.
- Heatmap visualization.

### 4.3 Trash

**Operations:** Read, restore.

**Features:**

- Soft-delete workflows.
- Restore functionality.
- Trash view.

## 5. Configuration

### 5.1 Settings

**Operations:** Read, write.

**Features:**

- Preferences.
- Sync setup.
- Local search model setup.
- Updater and What's New.
- Diagnostics where present.
- Encrypted sensitive settings for sync credentials and secrets.

### 5.2 Theme System

**Operations:** Configure, switch.

**Modes:** Light, dark, system.

## 6. Sync And Updater

### 6.1 Self-Hosted Encrypted Sync

**Features:**

- Device enrollment.
- Encrypted change push/pull.
- Encrypted image/video blob sync.
- Reconnect with sync passphrase.
- Sync-server compatibility.

### 6.2 OTA Updater

**Features:**

- Manual update checks.
- Auto-check preference.
- Available update state.
- Skip version.
- Download and install.

## 7. Not In V1

The following are not v1 product or command/API surfaces:

- Canvas.
- Bookmarks.
- Graph.
- Journal audio recording.
- Transcription.
- AI journal enrichment.
- Provider-key setup for AI/transcription.
- Transcription model management.
- Resource-link APIs.
- Bookmark metadata extraction.
- Search result types beyond journal entries and tasks.

Database migrations and sync compatibility handlers may still mention older entities. That compatibility should not be treated as a shipped feature.
