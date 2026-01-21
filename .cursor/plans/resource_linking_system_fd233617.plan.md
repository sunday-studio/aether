---
name: Resource Linking System
overview: Implement an Obsidian-style bidirectional linking system using `[[]]` syntax that allows linking between all resources (journal entries, tasks, goals, canvas, bookmarks) with autocomplete search and backlink display in popovers.
todos:
  - id: db-schema
    content: Create database migration for resource_links table with indexes
    status: pending
  - id: backend-models
    content: Add ResourceLink model to db/models.rs
    status: pending
  - id: link-repository
    content: Create link repository with CRUD operations and backlink queries
    status: pending
  - id: link-parser
    content: Create link parser utility to extract [[links]] from Lexical JSON content
    status: pending
  - id: tauri-commands
    content: Create Tauri commands for link operations (create, get backlinks, search)
    status: pending
  - id: lexical-link-node
    content: Create ResourceLinkNode and ResourceLinkPlugin for Lexical editor
    status: pending
  - id: autocomplete-component
    content: Create autocomplete component for [[link]] search and selection
    status: pending
  - id: link-rendering
    content: Create ResourceLink component for rendering clickable links
    status: pending
  - id: backlinks-popover
    content: Create BacklinksPopover component to display resources linking to current resource
    status: pending
  - id: integrate-journal
    content: Integrate ResourceLinkPlugin into journal editor
    status: pending
  - id: integrate-tasks
    content: Add link support to task description editor
    status: pending
  - id: integrate-goals
    content: Add link support to goal description editor
    status: pending
  - id: integrate-canvas
    content: Add link parsing and support to canvas nodes
    status: pending
  - id: sync-handlers
    content: Update all resource handlers to sync links when content is saved
    status: pending
  - id: api-client
    content: Add link API methods to frontend API client
    status: pending
---

# Resource Linking System Implementation

## Overview

This plan implements a bidirectional linking system similar to Obsidian's `[[]]` syntax, allowing users to create links between all resource types (journal entries, tasks, goals, canvas nodes, bookmarks) with autocomplete search and backlink visualization.

## Architecture

### Database Schema

Create a new migration file `009_add_resource_links.sql`:

- **`resource_links` table**: Stores bidirectional links between resources
  - `id` (TEXT PRIMARY KEY)
  - `source_type` (TEXT) - entry, task, goal, canvas, bookmark
  - `source_id` (TEXT) - ID of the source resource
  - `target_type` (TEXT) - entry, task, goal, canvas, bookmark
  - `target_id` (TEXT) - ID of the target resource
  - `link_text` (TEXT) - Optional display text for the link
  - `created_at` (TEXT)
  - Indexes on `(source_type, source_id)` and `(target_type, target_id)` for efficient backlink queries

### Backend Implementation

#### 1. Database Models (`desktop/src-tauri/src/db/models.rs`)

Add `ResourceLink` model:

```rust
pub struct ResourceLink {
    pub id: String,
    pub source_type: String,
    pub source_id: String,
    pub target_type: String,
    pub target_id: String,
    pub link_text: Option<String>,
    pub created_at: DateTime<Utc>,
}
```

#### 2. Link Repository (`desktop/src-tauri/src/db/repositories/link.rs`)

Create repository with methods:

- `create()` - Create a new link
- `find_by_source()` - Get all links from a source resource
- `find_by_target()` - Get all backlinks to a target resource
- `delete()` - Remove a link
- `delete_by_source()` - Remove all links from a source (when resource is deleted)
- `extract_links_from_content()` - Parse content to extract `[[]]` links

#### 3. Link Parser (`desktop/src-tauri/src/utils/link_parser.rs`)

Utility to:

- Parse Lexical JSON content to extract text
- Find `[[]]` patterns in text
- Match link text to resources using search/fuzzy matching
- Return structured link data

#### 4. Tauri Commands (`desktop/src-tauri/src/commands/link.rs`)

Commands:

- `create_link()` - Create a link between resources
- `get_backlinks()` - Get all resources linking to a target
- `get_outgoing_links()` - Get all links from a source
- `delete_link()` - Remove a link
- `search_linkable_resources()` - Search for resources to link (for autocomplete)

#### 5. Update Resource Handlers

Modify handlers to:

- Extract and sync links when content is saved (entries, tasks, goals, canvas, bookmarks)
- Delete links when resources are deleted
- Include link counts in resource queries

### Frontend Implementation

#### 1. Link Node for Lexical (`desktop/src/components/editor/plugins/resource-link-plugin/`)

Create new Lexical node and plugin:

- `ResourceLinkNode` - Custom Lexical node for `[[]]` links
- `ResourceLinkPlugin` - Plugin that:
  - Detects `[[` trigger
  - Shows autocomplete/search dropdown
  - Creates `ResourceLinkNode` when link is selected
  - Renders links with proper styling

#### 2. Autocomplete Component (`desktop/src/components/editor/plugins/resource-link-plugin/resource-link-autocomplete.tsx`)

- Typeahead menu similar to slash commands
- Searches across all resource types
- Shows resource type icon, name/title, and preview
- Filters results as user types

#### 3. Link Rendering (`desktop/src/components/shared/resource-link.tsx`)

Component that:

- Renders clickable link with resource name/title
- Shows resource type badge/icon
- Handles navigation to linked resource
- Opens backlink popover on click

#### 4. Backlink Popover (`desktop/src/components/shared/backlinks-popover.tsx`)

Popover component that:

- Fetches backlinks for a resource
- Displays list of resources linking to current resource
- Shows preview/snippet of link context
- Allows navigation to linking resources

#### 5. Integration Points

Update resource views to:

- **Journal Editor**: Add `ResourceLinkPlugin` to editor config
- **Task Description**: Support links in task description editor
- **Goal Description**: Support links in goal description editor
- **Canvas**: Parse canvas JSON for link nodes, support linking in canvas text nodes
- **Bookmarks**: Support links in bookmark notes/description (if applicable)

#### 6. API Client (`desktop/src/lib/api-client.ts`)

Add methods:

- `createLink()`
- `getBacklinks()`
- `getOutgoingLinks()`
- `deleteLink()`
- `searchLinkableResources()`

## Implementation Details

### Link Syntax Parsing

The `[[]]` syntax will be parsed as:

- `[[resource-name]]` - Fuzzy match by name/title
- `[[resource-type:resource-id]]` - Explicit ID (for programmatic links)
- Links are extracted from Lexical JSON by:

  1. Converting Lexical state to plain text
  2. Finding `[[...]]` patterns
  3. Matching to resources via search API
  4. Creating `ResourceLinkNode` instances

### Link Storage

Links are stored in two ways:

1. **In content**: As `ResourceLinkNode` in Lexical JSON (for rendering)
2. **In database**: As `resource_links` table rows (for queries and backlinks)

When content is saved:

- Extract all `ResourceLinkNode` instances
- Sync with database (create new, delete removed, update changed)

### Backlink Queries

Backlinks are queried efficiently using:

```sql
SELECT * FROM resource_links 
WHERE target_type = ? AND target_id = ?
```

### Resource Identification

Each resource type has:

- **Entry**: Use entry ID, display first line of content or "Untitled Entry"
- **Task**: Use task ID, display task title
- **Goal**: Use goal ID, display goal name
- **Canvas**: Use canvas ID, display canvas name
- **Bookmark**: Use bookmark ID, display bookmark title or URL

## Files to Create/Modify

### New Files

- `desktop/src-tauri/migrations/009_add_resource_links.sql`
- `desktop/src-tauri/src/db/repositories/link.rs`
- `desktop/src-tauri/src/utils/link_parser.rs`
- `desktop/src-tauri/src/commands/link.rs`
- `desktop/src/components/editor/plugins/resource-link-plugin/resource-link-node.tsx`
- `desktop/src/components/editor/plugins/resource-link-plugin/resource-link-plugin.tsx`
- `desktop/src/components/editor/plugins/resource-link-plugin/resource-link-autocomplete.tsx`
- `desktop/src/components/shared/resource-link.tsx`
- `desktop/src/components/shared/backlinks-popover.tsx`

### Modified Files

- `desktop/src-tauri/src/db/models.rs` - Add `ResourceLink` model
- `desktop/src-tauri/src/db/repositories/mod.rs` - Export link repository
- `desktop/src-tauri/src/db/schema.rs` - Add link table creation
- `desktop/src-tauri/src/lib.rs` - Register link commands
- `desktop/src-tauri/src/handlers/entry.rs` - Sync links on save
- `desktop/src-tauri/src/handlers/task.rs` - Sync links on save
- `desktop/src-tauri/src/handlers/goal.rs` - Sync links on save
- `desktop/src-tauri/src/handlers/canvas.rs` - Sync links on save
- `desktop/src-tauri/src/handlers/bookmark.rs` - Sync links on save
- `desktop/src/components/editor/editor.tsx` - Add `ResourceLinkPlugin`
- `desktop/src/features/journal/components/journal-editor.tsx` - Ensure plugin is active
- `desktop/src/features/tasks/components/task-item/task-item-description.tsx` - Add link support
- `desktop/src/lib/api-client.ts` - Add link API methods

## Testing Considerations

- Test link creation from all resource types
- Test backlink queries return correct results
- Test link deletion when resources are deleted
- Test autocomplete search accuracy
- Test link rendering in all contexts
- Test bidirectional link integrity

## Future Enhancements

- Link preview on hover
- Graph view of all links
- Link suggestions based on content similarity
- Link aliases/display names
- Link context snippets in backlinks