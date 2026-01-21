---
name: Bookmarking Service Implementation
overview: Build a local-first bookmarking service with URL metadata extraction, Chrome extension integration, and full search support. Bookmarks are stored separately from entries, with metadata-only extraction (title, description, images) using Open Graph data for all links, with local-first extraction and external API fallback.
todos:
  - id: db_schema
    content: Create migration 007_add_bookmarks_table.sql with bookmarks table, FTS index, tags relationship, and embedding column
    status: pending
  - id: bookmark_model
    content: Add Bookmark model to db/models.rs with all metadata fields
    status: pending
  - id: bookmark_repo
    content: Create db/repositories/bookmark.rs with CRUD operations, tag management, and embedding generation
    status: pending
  - id: metadata_extractor
    content: Create utils/metadata/extractor.rs for local HTML parsing (Open Graph, Twitter Cards, standard meta tags)
    status: pending
  - id: metadata_providers
    content: Create utils/metadata/providers.rs for external API fallback (LinkPreview, Microlink)
    status: pending
  - id: bookmark_commands
    content: Create commands/bookmark.rs with Tauri commands for bookmark CRUD and tag management
    status: pending
  - id: search_integration
    content: Add Bookmark to ResourceType enum and implement bookmark search in repositories/search.rs
    status: pending
  - id: register_commands
    content: Register bookmark commands in lib.rs and add to routeToCommand mapping in api-client.ts
    status: pending
  - id: bookmarks_domain
    content: Create features/bookmarks/bookmarks.domain.ts with TypeScript types
    status: pending
  - id: bookmarks_ui
    content: Implement bookmarks.view.tsx with grid/list view, filters, and bookmark cards
    status: pending
  - id: bookmark_form
    content: Create bookmark-form.tsx component for creating/editing bookmarks with metadata preview
    status: pending
  - id: chrome_manifest
    content: Create chrome-extension/manifest.json with required permissions and structure
    status: pending
  - id: chrome_popup
    content: Build chrome-extension/popup/popup.tsx with metadata preview and save functionality
    status: pending
  - id: chrome_background
    content: Implement chrome-extension/background.js for native messaging or HTTP communication with Tauri
    status: pending
  - id: chrome_content
    content: Create chrome-extension/content.js to extract page metadata from DOM
    status: pending
---

# Bookmarking Service Implementation Plan

## Overview

Build a comprehensive bookmarking service that allows users to save URLs with automatically extracted metadata, process them for search, and bookmark via Chrome extension. The service integrates with the existing search infrastructure and follows the local-first architecture.

## Architecture

```
┌─────────────────┐
│ Chrome Extension │
│  (Bookmark UI)   │
└────────┬─────────┘
         │
         ▼
┌─────────────────┐
│  Tauri Backend  │
│  ┌───────────┐  │
│  │ Metadata  │  │
│  │ Extractor │  │
│  └─────┬─────┘  │
│        │        │
│  ┌─────▼─────┐  │
│  │ Bookmarks │  │
│  │ Repository│  │
│  └─────┬─────┘  │
│        │        │
│  ┌─────▼─────┐  │
│  │  Search   │  │
│  │ Integration│ │
│  └───────────┘  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   LibSQL DB     │
│  - bookmarks    │
│  - bookmarks_fts│
│  - bookmark_tags│
└─────────────────┘
```

## Database Schema

### Migration: `007_add_bookmarks_table.sql`

Create a new `bookmarks` table separate from entries:

```sql
-- Bookmarks table
CREATE TABLE IF NOT EXISTS bookmarks (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL UNIQUE,
    title TEXT,
    description TEXT,
    image_url TEXT,
    favicon_url TEXT,
    site_name TEXT,
    author TEXT,
    published_at TEXT,
    content_type TEXT, -- 'article', 'video', 'tweet', 'image', etc.
    metadata_json TEXT, -- Full metadata as JSON for extensibility
    is_archived INTEGER NOT NULL DEFAULT 0,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_bookmarks_url ON bookmarks(url);
CREATE INDEX IF NOT EXISTS idx_bookmarks_created_at ON bookmarks(created_at);
CREATE INDEX IF NOT EXISTS idx_bookmarks_deleted_at ON bookmarks(deleted_at);
CREATE INDEX IF NOT EXISTS idx_bookmarks_content_type ON bookmarks(content_type);

-- FTS5 index for search
CREATE VIRTUAL TABLE IF NOT EXISTS bookmarks_fts USING fts5(
    title,
    description,
    site_name,
    author,
    tokenize='trigram',
    detail='column'
);

-- Triggers to keep FTS index in sync
CREATE TRIGGER IF NOT EXISTS bookmarks_fts_insert AFTER INSERT ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(rowid, title, description, site_name, author)
    VALUES (new.id, COALESCE(new.title, ''), COALESCE(new.description, ''), 
            COALESCE(new.site_name, ''), COALESCE(new.author, ''));
END;

CREATE TRIGGER IF NOT EXISTS bookmarks_fts_delete AFTER DELETE ON bookmarks BEGIN
    DELETE FROM bookmarks_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS bookmarks_fts_update AFTER UPDATE ON bookmarks BEGIN
    DELETE FROM bookmarks_fts WHERE rowid = old.id;
    INSERT INTO bookmarks_fts(rowid, title, description, site_name, author)
    VALUES (new.id, COALESCE(new.title, ''), COALESCE(new.description, ''), 
            COALESCE(new.site_name, ''), COALESCE(new.author, ''));
END;

-- Bookmark-Tag many-to-many relationship
CREATE TABLE IF NOT EXISTS bookmark_tags (
    bookmark_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (bookmark_id, tag_id),
    FOREIGN KEY (bookmark_id) REFERENCES bookmarks(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Add embedding column for semantic search
ALTER TABLE bookmarks ADD COLUMN embedding F32_BLOB(384);

-- Create vector index
CREATE INDEX IF NOT EXISTS bookmarks_embedding_idx 
    ON bookmarks(libsql_vector_idx(embedding, 'metric=cosine'));

-- Backfill existing bookmarks into FTS
INSERT INTO bookmarks_fts(rowid, title, description, site_name, author)
SELECT id, COALESCE(title, ''), COALESCE(description, ''), 
       COALESCE(site_name, ''), COALESCE(author, '')
FROM bookmarks WHERE deleted_at IS NULL;
```

## Backend Implementation

### 1. Database Models

**File:** `desktop/src-tauri/src/db/models.rs`

Add `Bookmark` model:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub id: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_json: Option<serde_json::Value>,
    pub is_archived: bool,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}
```

### 2. Metadata Extraction Service

**File:** `desktop/src-tauri/src/utils/metadata/mod.rs`

Create metadata extraction module with local-first approach:

```rust
pub mod extractor;
pub mod providers;

pub use extractor::MetadataExtractor;
```

**File:** `desktop/src-tauri/src/utils/metadata/extractor.rs`

Main extraction logic:

- Parse HTML using `scraper` crate
- Extract Open Graph tags (`og:title`, `og:description`, `og:image`, `og:site_name`, `og:type`, etc.)
- Extract Twitter Card tags (`twitter:title`, `twitter:description`, `twitter:image`, etc.) as fallback
- Extract standard meta tags (`title`, `description`, `author`) as final fallback
- Extract favicon from `<link rel="icon">` or default favicon.ico
- For all URLs (including social media), rely on Open Graph data provided by the site
- Fallback to external API if local extraction fails

**File:** `desktop/src-tauri/src/utils/metadata/providers.rs`

External API providers (fallback):

- LinkPreview API
- Microlink API
- Open Graph.io

### 3. Bookmarks Repository

**File:** `desktop/src-tauri/src/db/repositories/bookmark.rs`

CRUD operations for bookmarks:

- `create()` - Create bookmark with metadata
- `find_by_id()` - Get bookmark by ID
- `find_by_url()` - Get bookmark by URL (for duplicate detection)
- `find_all()` - List all bookmarks with filters
- `update()` - Update bookmark
- `delete()` - Soft delete bookmark
- `add_tags()` / `remove_tags()` - Tag management
- `generate_embedding()` - Generate embedding for semantic search

### 4. Bookmarks Commands

**File:** `desktop/src-tauri/src/commands/bookmark.rs`

Tauri commands:

- `create_bookmark(url: String)` - Create bookmark, extract metadata
- `get_bookmarks(filters)` - List bookmarks
- `get_bookmark_by_id(id: String)` - Get single bookmark
- `update_bookmark(id: String, updates)` - Update bookmark
- `delete_bookmark(id: String)` - Delete bookmark
- `add_tags_to_bookmark(id: String, tag_ids: Vec<String>)` - Add tags
- `remove_tags_from_bookmark(id: String, tag_ids: Vec<String>)` - Remove tags
- `extract_metadata(url: String)` - Extract metadata without saving (for preview)

### 5. Search Integration

**File:** `desktop/src-tauri/src/db/repositories/search.rs`

Add `Bookmark` to `ResourceType` enum and implement search:

- Add `Bookmark` variant to `SearchResult` enum
- Implement `search_fuzzy()` for bookmarks using `bookmarks_fts`
- Implement `search_semantic()` for bookmarks using embeddings
- Update `search_internal()` to include bookmarks

## Chrome Extension

### Structure

```
desktop/chrome-extension/
├── manifest.json
├── background.js
├── content.js
├── popup/
│   ├── popup.html
│   ├── popup.tsx
│   └── popup.css
└── icons/
    └── icon-*.png
```

### manifest.json

```json
{
  "manifest_version": 3,
  "name": "Aether Bookmarks",
  "version": "1.0.0",
  "description": "Save bookmarks to Aether",
  "permissions": [
    "activeTab",
    "storage"
  ],
  "host_permissions": [
    "http://localhost/*",
    "https://*/*"
  ],
  "background": {
    "service_worker": "background.js"
  },
  "action": {
    "default_popup": "popup/popup.html",
    "default_icon": {
      "16": "icons/icon-16.png",
      "48": "icons/icon-48.png",
      "128": "icons/icon-128.png"
    }
  },
  "content_scripts": [{
    "matches": ["<all_urls>"],
    "js": ["content.js"]
  }]
}
```

### Features

1. **Popup UI** (`popup/popup.tsx`):

   - Show current page URL
   - Display extracted metadata preview
   - Allow editing title/description
   - Tag selection
   - Save button

2. **Background Service** (`background.js`):

   - Listen for extension icon clicks
   - Communicate with Tauri backend via native messaging or HTTP
   - Handle bookmark creation

3. **Content Script** (`content.js`):

   - Extract page metadata from DOM
   - Send to popup for preview

### Communication with Tauri

Since Chrome extensions can't directly call Tauri commands, options:

1. **Native Messaging** (recommended):

   - Tauri app registers as native messaging host
   - Extension communicates via `chrome.runtime.sendNativeMessage()`

2. **HTTP Server** (simpler):

   - Tauri app runs local HTTP server (port 9119)
   - Extension makes HTTP requests
   - Reuse existing HTTP infrastructure if available

## Frontend Implementation

### 1. Bookmarks Domain

**File:** `desktop/src/features/bookmarks/bookmarks.domain.ts`

TypeScript types and domain logic:

```typescript
export interface Bookmark {
  id: string;
  url: string;
  title?: string;
  description?: string;
  imageUrl?: string;
  faviconUrl?: string;
  siteName?: string;
  author?: string;
  publishedAt?: string;
  contentType?: string;
  metadataJson?: Record<string, unknown>;
  isArchived: boolean;
  isDeleted: boolean;
  createdAt: string;
  updatedAt: string;
  deletedAt?: string;
  tags?: Tag[];
}
```

### 2. Bookmarks View

**File:** `desktop/src/features/bookmarks/bookmarks.view.tsx`

Main bookmarks UI:

- Grid/List view toggle
- Filter by tags, content type, date
- Search integration
- Bookmark cards with:
  - Thumbnail image
  - Title and description
  - Site name
  - Tags
  - Actions (edit, delete, archive, open)

### 3. Bookmark Form/Editor

**File:** `desktop/src/features/bookmarks/components/bookmark-form.tsx`

Form for creating/editing bookmarks:

- URL input with validation
- Metadata preview
- Title/description editing
- Tag selector
- Save/Cancel actions

### 4. API Integration

**File:** `desktop/src/lib/api-client.ts`

Add bookmark routes to `routeToCommand` mapping:

```typescript
"GET /v1/bookmarks": "get_bookmarks",
"POST /v1/bookmarks": "create_bookmark",
"GET /v1/bookmarks/:id": "get_bookmark_by_id",
"PUT /v1/bookmarks/:id": "update_bookmark",
"DELETE /v1/bookmarks/:id": "delete_bookmark",
"POST /v1/bookmarks/:id/tags": "add_tags_to_bookmark",
"DELETE /v1/bookmarks/:id/tags": "remove_tags_from_bookmark",
```

## Metadata Extraction Strategy

All URLs (including social media) use the same extraction approach:

1. **Primary**: Extract Open Graph tags from HTML

   - Most modern sites (Twitter, YouTube, GitHub, etc.) provide Open Graph metadata
   - Standardized format: `og:title`, `og:description`, `og:image`, `og:site_name`, `og:type`

2. **Fallback 1**: Extract Twitter Card tags if Open Graph not available

   - `twitter:title`, `twitter:description`, `twitter:image`

3. **Fallback 2**: Extract standard HTML meta tags

   - `<title>`, `<meta name="description">`, `<meta name="author">`

4. **Fallback 3**: External API (LinkPreview, Microlink)

   - Only if local extraction fails or returns insufficient data

## Dependencies

### Cargo.toml additions

```toml
# HTML parsing
scraper = "0.20"
html5ever = "0.27"

# URL parsing
url = "2.5"

# JSON handling (already have serde_json)
```

### package.json additions (for Chrome extension)

```json
{
  "devDependencies": {
    "@types/chrome": "^0.0.268"
  }
}
```

## Implementation Milestones

## Milestone 1: Backend Implementation

### Phase 1.1: Database Schema & Models

**Tasks:**

- Create migration `007_add_bookmarks_table.sql` with:
  - `bookmarks` table with all metadata fields
  - FTS5 index (`bookmarks_fts`) with triggers
  - `bookmark_tags` junction table
  - Embedding column and vector index
- Add `Bookmark` model to `db/models.rs`
- Update `db/schema.rs` if needed

**Files:**

- `desktop/src-tauri/migrations/007_add_bookmarks_table.sql`
- `desktop/src-tauri/src/db/models.rs`

### Phase 1.2: Bookmarks Repository

**Tasks:**

- Create `db/repositories/bookmark.rs` with:
  - `create()` - Create bookmark with metadata
  - `find_by_id()` - Get bookmark by ID
  - `find_by_url()` - Get bookmark by URL (duplicate detection)
  - `find_all()` - List bookmarks with filters (archived, deleted, tags, date range)
  - `update()` - Update bookmark
  - `delete()` - Soft delete bookmark
  - `add_tags()` / `remove_tags()` - Tag management
  - `generate_embedding()` - Generate embedding for semantic search
- Add bookmark repository to `db/repositories/mod.rs`

**Files:**

- `desktop/src-tauri/src/db/repositories/bookmark.rs`
- `desktop/src-tauri/src/db/repositories/mod.rs`

### Phase 1.3: Metadata Extraction

**Tasks:**

- Create `utils/metadata/mod.rs` module
- Create `utils/metadata/extractor.rs`:
  - Parse HTML using `scraper` crate
  - Extract Open Graph tags (primary)
  - Extract Twitter Card tags (fallback 1)
  - Extract standard meta tags (fallback 2)
  - Extract favicon
  - Return structured metadata
- Create `utils/metadata/providers.rs`:
  - External API fallback (LinkPreview, Microlink)
  - Error handling and retries
- Add metadata module to `utils/mod.rs`

**Files:**

- `desktop/src-tauri/src/utils/metadata/mod.rs`
- `desktop/src-tauri/src/utils/metadata/extractor.rs`
- `desktop/src-tauri/src/utils/metadata/providers.rs`
- `desktop/src-tauri/src/utils/mod.rs`
- `desktop/src-tauri/Cargo.toml` (add `scraper` dependency)

### Phase 1.4: Bookmarks Commands

**Tasks:**

- Create `commands/bookmark.rs` with Tauri commands:
  - `create_bookmark(url: String, tags: Option<Vec<String>>)` - Create bookmark, extract metadata
  - `get_bookmarks(filters)` - List bookmarks with filters
  - `get_bookmark_by_id(id: String)` - Get single bookmark
  - `update_bookmark(id: String, updates)` - Update bookmark
  - `delete_bookmark(id: String)` - Delete bookmark
  - `add_tags_to_bookmark(id: String, tag_ids: Vec<String>)` - Add tags
  - `remove_tags_from_bookmark(id: String, tag_ids: Vec<String>)` - Remove tags
  - `extract_metadata(url: String)` - Extract metadata without saving (for preview)
- Register commands in `lib.rs`
- Add routes to `api-client.ts` routeToCommand mapping

**Files:**

- `desktop/src-tauri/src/commands/bookmark.rs`
- `desktop/src-tauri/src/commands/mod.rs`
- `desktop/src-tauri/src/lib.rs`
- `desktop/src/lib/api-client.ts`

### Phase 1.5: Search Integration

**Tasks:**

- Add `Bookmark` variant to `ResourceType` enum in `repositories/search.rs`
- Add `Bookmark` variant to `SearchResult` enum
- Implement `search_fuzzy()` for bookmarks using `bookmarks_fts`
- Implement `search_semantic()` for bookmarks using embeddings
- Update `search_internal()` to include bookmarks
- Update search command to handle bookmark resource type

**Files:**

- `desktop/src-tauri/src/db/repositories/search.rs`
- `desktop/src-tauri/src/commands/search.rs`

## Milestone 2: Frontend Implementation

### Phase 2.1: Domain & Types

**Tasks:**

- Create `features/bookmarks/bookmarks.domain.ts`:
  - Define `Bookmark` TypeScript interface
  - Define bookmark filter types
  - Define bookmark form types
- Generate OpenAPI types (via Orval) for bookmarks

**Files:**

- `desktop/src/features/bookmarks/bookmarks.domain.ts`
- Run `npm run generate:sdk` to generate types

### Phase 2.2: Bookmarks View

**Tasks:**

- Implement `bookmarks.view.tsx`:
  - Grid/List view toggle
  - Filter by tags, content type, date range, archived status
  - Search integration (use existing search)
  - Infinite scroll or pagination
  - Empty state
- Create `components/bookmark-card.tsx`:
  - Display thumbnail image
  - Show title, description, site name
  - Display tags
  - Actions (edit, delete, archive, open in browser)
- Add bookmarks route to router

**Files:**

- `desktop/src/features/bookmarks/bookmarks.view.tsx`
- `desktop/src/features/bookmarks/components/bookmark-card.tsx`
- `desktop/src/features/router.tsx`

### Phase 2.3: Bookmark Form

**Tasks:**

- Create `components/bookmark-form.tsx`:
  - URL input with validation
  - Metadata preview (title, description, image)
  - Editable title/description fields
  - Tag selector (reuse existing tag selector component)
  - Save/Cancel actions
  - Loading state during metadata extraction
- Create `components/bookmark-form-dialog.tsx` (wrapper with modal)
- Integrate with TanStack Query for mutations

**Files:**

- `desktop/src/features/bookmarks/components/bookmark-form.tsx`
- `desktop/src/features/bookmarks/components/bookmark-form-dialog.tsx`

### Phase 2.4: Chrome Extension

**Tasks:**

- Create extension structure:
  - `chrome-extension/manifest.json` with permissions
  - `chrome-extension/popup/popup.html` and `popup.tsx`
  - `chrome-extension/background.js` (service worker)
  - `chrome-extension/content.js` (content script)
  - Extension icons
- Implement popup UI:
  - Show current page URL
  - Display extracted metadata preview
  - Allow editing title/description
  - Tag selection
  - Save button
- Implement communication:
  - Option 1: Native messaging (Tauri registers as native host)
  - Option 2: HTTP server (Tauri runs local HTTP server on port 9119)
- Test bookmarking flow from extension

**Files:**

- `desktop/chrome-extension/manifest.json`
- `desktop/chrome-extension/popup/popup.html`
- `desktop/chrome-extension/popup/popup.tsx`
- `desktop/chrome-extension/popup/popup.css`
- `desktop/chrome-extension/background.js`
- `desktop/chrome-extension/content.js`
- `desktop/chrome-extension/icons/icon-*.png`

### Phase 2.5: Polish & Integration

**Tasks:**

- Error handling:
  - Invalid URLs
  - Network failures during metadata extraction
  - Duplicate URL detection
- Loading states:
  - Metadata extraction spinner
  - Save button loading state
- Performance:
  - Lazy load bookmark images
  - Virtual scrolling for large lists
  - Debounce search input
- Testing:
  - Test metadata extraction for various sites
  - Test duplicate URL handling
  - Test search integration
  - Test Chrome extension communication
  - Test error cases

## Key Files Summary

### Backend Files (Milestone 1)

**New:**

- `desktop/src-tauri/migrations/007_add_bookmarks_table.sql`
- `desktop/src-tauri/src/db/repositories/bookmark.rs`
- `desktop/src-tauri/src/commands/bookmark.rs`
- `desktop/src-tauri/src/utils/metadata/mod.rs`
- `desktop/src-tauri/src/utils/metadata/extractor.rs`
- `desktop/src-tauri/src/utils/metadata/providers.rs`

**Modified:**

- `desktop/src-tauri/src/db/models.rs` - Add Bookmark model
- `desktop/src-tauri/src/db/repositories/mod.rs` - Export bookmark repository
- `desktop/src-tauri/src/db/repositories/search.rs` - Add bookmark search
- `desktop/src-tauri/src/commands/mod.rs` - Export bookmark commands
- `desktop/src-tauri/src/lib.rs` - Register bookmark commands
- `desktop/src-tauri/src/utils/mod.rs` - Export metadata module
- `desktop/src-tauri/Cargo.toml` - Add `scraper` dependency
- `desktop/src/lib/api-client.ts` - Add bookmark routes

### Frontend Files (Milestone 2)

**New:**

- `desktop/src/features/bookmarks/bookmarks.domain.ts`
- `desktop/src/features/bookmarks/components/bookmark-form.tsx`
- `desktop/src/features/bookmarks/components/bookmark-form-dialog.tsx`
- `desktop/src/features/bookmarks/components/bookmark-card.tsx`
- `desktop/chrome-extension/manifest.json`
- `desktop/chrome-extension/background.js`
- `desktop/chrome-extension/content.js`
- `desktop/chrome-extension/popup/popup.html`
- `desktop/chrome-extension/popup/popup.tsx`
- `desktop/chrome-extension/popup/popup.css`
- `desktop/chrome-extension/icons/icon-*.png`

**Modified:**

- `desktop/src/features/bookmarks/bookmarks.view.tsx` - Implement UI
- `desktop/src/features/router.tsx` - Add bookmarks route

## Testing Considerations

### Backend Testing

- Test metadata extraction for various sites (regular pages, Twitter, YouTube, GitHub, etc.)
- Test duplicate URL handling
- Test search integration (fuzzy and semantic)
- Test error cases (invalid URLs, network failures, malformed HTML)
- Test embedding generation for bookmarks
- Test tag management operations

### Frontend Testing

- Test bookmark creation flow
- Test bookmark editing
- Test filtering and search
- Test Chrome extension communication
- Test error states and loading states
- Test duplicate detection UI
- Test bookmark deletion and archiving