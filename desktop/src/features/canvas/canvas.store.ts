import { proxy } from "valtio";
import type { CanvasNode, Edge, Point, CanvasState } from "./types";

interface CanvasStore {
	// Canvas data
	nodes: CanvasNode[];
	edges: Edge[];

	// Viewport
	zoom: number;
	panX: number;
	panY: number;

	// Interaction
	selectedNodeIds: Set<string>;
	isDragging: boolean;
	dragStart: Point | null;

	// History (for undo/redo)
	history: CanvasState[];
	historyIndex: number;

	// Actions
	setNodes: (nodes: CanvasNode[]) => void;
	setEdges: (edges: Edge[]) => void;
	addNode: (node: CanvasNode) => void;
	updateNode: (id: string, updates: Partial<CanvasNode>) => void;
	deleteNode: (id: string) => void;
	addEdge: (edge: Edge) => void;
	updateEdge: (id: string, updates: Partial<Edge>) => void;
	deleteEdge: (id: string) => void;
	setViewport: (zoom: number, panX: number, panY: number) => void;
	pan: (deltaX: number, deltaY: number) => void;
	zoomTo: (zoom: number, centerX?: number, centerY?: number) => void;
	selectNode: (id: string, addToSelection?: boolean) => void;
	deselectNode: (id: string) => void;
	clearSelection: () => void;
	startDrag: (x: number, y: number) => void;
	updateDrag: (x: number, y: number) => void;
	endDrag: () => void;
	saveState: () => void;
	undo: () => void;
	redo: () => void;
	canUndo: () => boolean;
	canRedo: () => boolean;
}

const MAX_HISTORY = 50;

function createCanvasStore(): CanvasStore {
	const store = proxy<CanvasStore>({
		nodes: [],
		edges: [],
		zoom: 1,
		panX: 0,
		panY: 0,
		selectedNodeIds: new Set(),
		isDragging: false,
		dragStart: null,
		history: [],
		historyIndex: -1,

		setNodes: (nodes) => {
			store.nodes = nodes;
		},

		setEdges: (edges) => {
			store.edges = edges;
		},

		addNode: (node) => {
			store.nodes.push(node);
			store.saveState();
		},

		updateNode: (id, updates) => {
			const index = store.nodes.findIndex((n) => n.id === id);
			if (index !== -1) {
				store.nodes[index] = { ...store.nodes[index], ...updates };
				store.saveState();
			}
		},

		deleteNode: (id) => {
			store.nodes = store.nodes.filter((n) => n.id !== id);
			store.edges = store.edges.filter(
				(e) => e.fromNode !== id && e.toNode !== id,
			);
			store.selectedNodeIds.delete(id);
			store.saveState();
		},

		addEdge: (edge) => {
			store.edges.push(edge);
			store.saveState();
		},

		updateEdge: (id, updates) => {
			const index = store.edges.findIndex((e) => e.id === id);
			if (index !== -1) {
				store.edges[index] = { ...store.edges[index], ...updates };
				store.saveState();
			}
		},

		deleteEdge: (id) => {
			store.edges = store.edges.filter((e) => e.id !== id);
			store.saveState();
		},

		setViewport: (zoom, panX, panY) => {
			store.zoom = zoom;
			store.panX = panX;
			store.panY = panY;
		},

		pan: (deltaX, deltaY) => {
			store.panX += deltaX;
			store.panY += deltaY;
		},

		zoomTo: (zoom, centerX, centerY) => {
			const oldZoom = store.zoom;
			store.zoom = Math.max(0.1, Math.min(5, zoom));

			if (centerX !== undefined && centerY !== undefined) {
				// Zoom towards the center point
				const scale = store.zoom / oldZoom;
				store.panX = centerX - (centerX - store.panX) * scale;
				store.panY = centerY - (centerY - store.panY) * scale;
			}
		},

		selectNode: (id, addToSelection = false) => {
			if (addToSelection) {
				store.selectedNodeIds.add(id);
			} else {
				store.selectedNodeIds.clear();
				store.selectedNodeIds.add(id);
			}
		},

		deselectNode: (id) => {
			store.selectedNodeIds.delete(id);
		},

		clearSelection: () => {
			store.selectedNodeIds.clear();
		},

		startDrag: (x, y) => {
			store.isDragging = true;
			store.dragStart = { x, y };
		},

		updateDrag: (x, y) => {
			if (!store.dragStart) return;

			const deltaX = x - store.dragStart.x;
			const deltaY = y - store.dragStart.y;

			// Move selected nodes
			store.selectedNodeIds.forEach((nodeId) => {
				const node = store.nodes.find((n) => n.id === nodeId);
				if (node) {
					node.x += deltaX / store.zoom;
					node.y += deltaY / store.zoom;
				}
			});

			store.dragStart = { x, y };
		},

		endDrag: () => {
			if (store.isDragging) {
				store.isDragging = false;
				store.dragStart = null;
				store.saveState();
			}
		},

		saveState: () => {
			const state: CanvasState = {
				nodes: [...store.nodes],
				edges: [...store.edges],
				zoom: store.zoom,
				panX: store.panX,
				panY: store.panY,
				selectedNodeIds: new Set(store.selectedNodeIds),
				isDragging: store.isDragging,
				dragStart: store.dragStart,
			};

			// Remove any states after current index (when undoing and then making a change)
			if (store.historyIndex < store.history.length - 1) {
				store.history = store.history.slice(0, store.historyIndex + 1);
			}

			store.history.push(state);
			store.historyIndex = store.history.length - 1;

			// Limit history size
			if (store.history.length > MAX_HISTORY) {
				store.history.shift();
				store.historyIndex = store.history.length - 1;
			}
		},

		undo: () => {
			if (store.canUndo()) {
				store.historyIndex--;
				const state = store.history[store.historyIndex];
				if (state) {
					store.nodes = [...state.nodes];
					store.edges = [...state.edges];
					store.zoom = state.zoom;
					store.panX = state.panX;
					store.panY = state.panY;
					store.selectedNodeIds = new Set(state.selectedNodeIds);
				}
			}
		},

		redo: () => {
			if (store.canRedo()) {
				store.historyIndex++;
				const state = store.history[store.historyIndex];
				if (state) {
					store.nodes = [...state.nodes];
					store.edges = [...state.edges];
					store.zoom = state.zoom;
					store.panX = state.panX;
					store.panY = state.panY;
					store.selectedNodeIds = new Set(state.selectedNodeIds);
				}
			}
		},

		canUndo: () => {
			return store.historyIndex > 0;
		},

		canRedo: () => {
			return store.historyIndex < store.history.length - 1;
		},
	});

	return store;
}

export const canvasStore = createCanvasStore();
