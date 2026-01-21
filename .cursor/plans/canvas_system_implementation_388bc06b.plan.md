---
name: Canvas System Implementation
overview: Build a JSON Canvas-compliant canvas system with pan/zoom, text/file/link nodes, edges, drag-drop, selection, and basic undo/redo. Support multiple canvases with full CRUD operations.
todos:
  - id: db-schema
    content: Create database migration for canvases table (007_add_canvas_tables.sql)
    status: pending
  - id: backend-models
    content: Add Canvas model to db/models.rs
    status: pending
  - id: backend-repository
    content: Create canvas repository (db/repositories/canvas.rs) with CRUD operations
    status: pending
  - id: backend-commands
    content: Create Tauri commands (commands/canvas.rs) and register in lib.rs
    status: pending
  - id: frontend-types
    content: Create TypeScript types for JSON Canvas spec (canvas/types.ts)
    status: pending
  - id: frontend-store
    content: Create Zustand store for canvas state (canvas/canvas.store.ts)
    status: pending
  - id: frontend-api-hooks
    content: Create React Query hooks for canvas API (canvas/hooks/use-canvas-api.ts)
    status: pending
  - id: install-konva
    content: Install react-konva and konva dependencies
    status: pending
  - id: canvas-viewport
    content: Create main canvas viewport component with pan/zoom (canvas/components/canvas-viewport.tsx)
    status: pending
  - id: node-components
    content: Create node components (text-node, file-node, link-node)
    status: pending
  - id: edge-renderer
    content: Create edge rendering component with connection points
    status: pending
  - id: drag-drop
    content: Implement node dragging and selection system
    status: pending
  - id: canvas-toolbar
    content: Create toolbar for adding nodes and canvas actions
    status: pending
  - id: canvas-management
    content: Create canvas list sidebar and header with name editing
    status: pending
  - id: undo-redo
    content: Implement basic undo/redo system with history
    status: pending
  - id: json-import-export
    content: Implement JSON Canvas import/export functionality
    status: pending
  - id: update-canvas-view
    content: Replace placeholder CanvasView with full implementation
    status: pending
---

# Canvas System Implementation Plan

## Overview

Build a JSON Canvas-compliant canvas system similar to FigJam, supporting multiple canvases with medium-level functionality: pan/zoom, text/file/link nodes, edges, drag-drop, selection, and basic undo/redo.

## Architecture

### Data Model

**Canvas Storage:**

- Store canvas data as JSON in database (following JSON Canvas spec)
- Each canvas is a separate document with metadata
- Nodes and edges stored as JSON arrays in the canvas document

**Database Schema:**

```sql
CREATE TABLE canvases (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    canvas_data TEXT NOT NULL, -- JSON Canvas format
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);
```

### Tech Stack Decisions

**Frontend Canvas Library:**

- Use **react-konva** or **@react-spring/konva** for canvas rendering
- Alternative: Custom SVG-based solution (lighter but more work)
- Konva provides: pan/zoom, drag-drop, selection, rendering performance

**State Management:**

- Use Zustand for canvas state (nodes, edges, viewport, selection)
- React Query for server state (canvas CRUD operations)

## Implementation Milestones

## Milestone 1: Backend Implementation

### Backend Phase 1: Database Schema

**Files to create:**

- `desktop/src-tauri/migrations/007_add_canvas_tables.sql` - Database schema

**Tasks:**

- Create `canvases` table with id, name, canvas_data (JSON), timestamps, and soft delete
- Add migration file following existing migration pattern

### Backend Phase 2: Data Models

**Files to modify:**

- `desktop/src-tauri/src/db/models.rs` - Add Canvas model

**Tasks:**

- Define Canvas struct with serde serialization
- Implement FromRow trait for database mapping
- Add validation for canvas data JSON structure

### Backend Phase 3: Repository Layer

**Files to create:**

- `desktop/src-tauri/src/db/repositories/canvas.rs` - Canvas repository

**Tasks:**

- Implement CRUD operations (create, read, update, delete)
- Add methods for listing all canvases
- Implement soft delete functionality
- Handle JSON canvas data serialization/deserialization

### Backend Phase 4: Tauri Commands

**Files to create/modify:**

- `desktop/src-tauri/src/commands/canvas.rs` - Tauri commands
- `desktop/src-tauri/src/lib.rs` - Register canvas commands

**Commands to implement:**

- `get_canvases()` - List all canvases
- `get_canvas_by_id(id)` - Get single canvas
- `create_canvas(name, canvas_data?)` - Create new canvas
- `update_canvas(id, name?, canvas_data?)` - Update canvas
- `delete_canvas(id)` - Soft delete canvas

**Tasks:**

- Create command handlers with error handling
- Register commands in lib.rs
- Add input validation
- Return proper error types

---

## Milestone 2: Frontend Implementation

### Frontend Phase 1: Core Infrastructure

**Files to create:**

- `desktop/src/features/canvas/types.ts` - TypeScript types for JSON Canvas
- `desktop/src/features/canvas/canvas.store.ts` - Zustand store for canvas state
- `desktop/src/features/canvas/hooks/use-canvas-api.ts` - React Query hooks

**Tasks:**

- Define TypeScript types matching JSON Canvas spec
- Create Zustand store with canvas state management
- Implement React Query hooks for API calls
- Set up store structure for nodes, edges, viewport, selection, and history

**Canvas Store Structure:**

```typescript
interface CanvasStore {
  // Canvas data
  nodes: Node[]
  edges: Edge[]
  
  // Viewport
  zoom: number
  panX: number
  panY: number
  
  // Interaction
  selectedNodeIds: Set<string>
  isDragging: boolean
  dragStart: { x: number, y: number } | null
  
  // History (for undo/redo)
  history: CanvasState[]
  historyIndex: number
  
  // Actions
  setNodes, setEdges, addNode, updateNode, deleteNode
  setEdges, addEdge, updateEdge, deleteEdge
  setViewport, pan, zoom
  selectNode, deselectNode, clearSelection
  startDrag, updateDrag, endDrag
  undo, redo
}
```

### Frontend Phase 2: Canvas Rendering & Viewport

**Files to create:**

- `desktop/src/features/canvas/components/canvas-viewport.tsx` - Main viewport with pan/zoom
- Install react-konva and konva dependencies

**Tasks:**

- Install `react-konva`, `konva`, and `@types/konva`
- Create main canvas viewport component
- Implement pan functionality (Space+drag or middle mouse drag)
- Implement zoom functionality (mouse wheel, pinch gestures)
- Set up coordinate transformation system

### Frontend Phase 3: Node Components

**Files to create:**

- `desktop/src/features/canvas/components/node-renderer.tsx` - Render individual nodes
- `desktop/src/features/canvas/components/text-node.tsx` - Text node component
- `desktop/src/features/canvas/components/file-node.tsx` - File node component
- `desktop/src/features/canvas/components/link-node.tsx` - Link node component

**Tasks:**

- Create base node renderer component
- Implement text node with markdown support
- Implement file node with icon/preview
- Implement link node with URL display
- Style nodes according to JSON Canvas spec

### Frontend Phase 4: Edges & Selection

**Files to create:**

- `desktop/src/features/canvas/components/edge-renderer.tsx` - Render edges/connections
- `desktop/src/features/canvas/components/selection-box.tsx` - Multi-select box

**Tasks:**

- Render edges between nodes with connection points
- Implement edge creation (click node handle, drag to another node)
- Add selection system (click, Shift+click, drag box)
- Visual feedback for selected nodes

### Frontend Phase 5: Node Interaction & Editing

**Files to create:**

- `desktop/src/features/canvas/components/canvas-toolbar.tsx` - Toolbar for adding nodes
- `desktop/src/features/canvas/components/node-editor.tsx` - Inline editing for text nodes
- `desktop/src/features/canvas/components/file-picker.tsx` - File selection for file nodes

**Tasks:**

- Implement node dragging system
- Add toolbar with node creation buttons
- Implement inline editing for text nodes (double-click)
- Add file picker for file nodes
- Add URL editor for link nodes

### Frontend Phase 6: Canvas Management UI

**Files to modify/create:**

- `desktop/src/features/canvas/canvas.view.tsx` - Main canvas view (replace placeholder)
- `desktop/src/features/canvas/components/canvas-list.tsx` - List of canvases (sidebar)
- `desktop/src/features/canvas/components/canvas-header.tsx` - Canvas name, save indicator

**Tasks:**

- Replace placeholder CanvasView with full implementation
- Create canvas list sidebar (like document list)
- Add create new canvas button
- Implement canvas name editing
- Add auto-save indicator
- Add delete canvas action

### Frontend Phase 7: Undo/Redo System

**Files to create:**

- `desktop/src/features/canvas/hooks/use-undo-redo.ts` - Undo/redo logic

**Tasks:**

- Store canvas state snapshots in Zustand store
- Limit history to ~50 states for memory efficiency
- Implement keyboard shortcuts: Cmd+Z (undo), Cmd+Shift+Z (redo)
- Clear history on canvas load
- Add undo/redo actions to store

### Frontend Phase 8: JSON Canvas Import/Export

**Files to create:**

- `desktop/src/features/canvas/utils/json-canvas.ts` - Import/export utilities
- `desktop/src/features/canvas/components/export-dialog.tsx` - Export to JSON file
- `desktop/src/features/canvas/components/import-dialog.tsx` - Import from JSON file

**Tasks:**

- Implement export to `.canvas` JSON file (JSON Canvas 1.0 spec)
- Implement import from `.canvas` files
- Validate JSON structure on import
- Create export/import dialogs
- Handle file I/O operations

## Technical Details

### Canvas Library Choice: react-konva

**Why react-konva:**

- Mature, well-documented
- Good performance for many nodes
- Built-in pan/zoom support
- Easy drag-drop implementation
- Active community

**Installation:**

```bash
npm install react-konva konva
npm install --save-dev @types/konva
```

### Viewport Coordinate System

- Canvas uses pixel coordinates
- Viewport transforms: `scale` (zoom) and `translate` (pan)
- Convert screen coordinates to canvas coordinates: `(screenX - panX) / zoom`

### Node Rendering

- Text nodes: Render markdown text with basic styling
- File nodes: Show file icon + filename, click to open
- Link nodes: Show URL preview with favicon if available

### Edge Rendering

- Use Konva Line or Arrow for edges
- Calculate connection points based on node position and side
- Support arrow heads at endpoints
- Optional labels on edges

### Performance Considerations

- Virtual rendering: Only render nodes in viewport + buffer
- Debounce auto-save (save after 2s of inactivity)
- Limit undo history size
- Use React.memo for node components

## File Structure

```
desktop/src/features/canvas/
├── canvas.view.tsx              # Main canvas view
├── canvas.store.ts              # Zustand store
├── types.ts                     # TypeScript types
├── hooks/
│   ├── use-canvas-api.ts       # React Query hooks
│   ├── use-canvas-interactions.ts # Pan/zoom/drag hooks
│   └── use-undo-redo.ts        # Undo/redo logic
├── components/
│   ├── canvas-viewport.tsx     # Main viewport
│   ├── canvas-toolbar.tsx      # Toolbar
│   ├── canvas-header.tsx       # Header with name
│   ├── canvas-list.tsx         # Sidebar list
│   ├── node-renderer.tsx       # Node wrapper
│   ├── text-node.tsx           # Text node
│   ├── file-node.tsx           # File node
│   ├── link-node.tsx           # Link node
│   ├── edge-renderer.tsx       # Edge rendering
│   ├── selection-box.tsx       # Multi-select
│   ├── node-editor.tsx         # Inline editing
│   ├── export-dialog.tsx       # Export UI
│   └── import-dialog.tsx       # Import UI
└── utils/
    ├── json-canvas.ts          # Import/export
    ├── coordinates.ts          # Coordinate transforms
    └── node-utils.ts           # Node helpers
```

## Dependencies to Add

**Frontend:**

- `react-konva` - Canvas rendering
- `konva` - Canvas library
- `@types/konva` - TypeScript types

**Backend:**

- No new dependencies needed (uses existing libsql, serde, etc.)

## Testing Considerations

- Test JSON Canvas import/export with sample files
- Test pan/zoom at various zoom levels
- Test node dragging and edge creation
- Test undo/redo with various operations
- Test canvas CRUD operations

## Future Enhancements (Out of Scope)

- Group nodes (container for other nodes)
- Node resizing
- Node rotation
- Advanced edge routing (curved, smart paths)
- Collaboration features
- Templates
- Node search/filter
- Canvas sharing

## Success Criteria

1. Users can create multiple canvases
2. Users can add text/file/link nodes and position them
3. Users can create edges between nodes
4. Users can pan and zoom the canvas
5. Users can drag nodes around
6. Users can select single/multiple nodes
7. Users can undo/redo operations
8. Canvas data persists to database
9. Users can export/import JSON Canvas files
10. UI is responsive and performant with 100+ nodes