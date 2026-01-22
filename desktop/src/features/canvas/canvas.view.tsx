import { useState, useEffect, useRef } from "react";
import { useSnapshot } from "valtio";
import { useCanvas, useUpdateCanvas } from "./hooks/use-canvas-api";
import { canvasStore } from "./canvas.store";
import { CanvasViewport } from "./components/canvas-viewport";
import { CanvasHeader } from "./components/canvas-header";
import { CanvasList } from "./components/canvas-list";
import { CanvasToolbar } from "./components/canvas-toolbar";
import type { Canvas } from "./types";
import { useDebounce } from "use-debounce";

export const CanvasView = () => {
	const [selectedCanvasId, setSelectedCanvasId] = useState<string | null>(null);
	const [viewportSize, setViewportSize] = useState({ width: 0, height: 0 });
	const containerRef = useRef<HTMLDivElement>(null);
	const { data: canvas } = useCanvas(selectedCanvasId);
	const { mutate: updateCanvas } = useUpdateCanvas();
	const store = useSnapshot(canvasStore);
	const [debouncedNodes] = useDebounce(store.nodes, 2000);
	const [debouncedEdges] = useDebounce(store.edges, 2000);

	// Update viewport size
	useEffect(() => {
		const updateSize = () => {
			if (containerRef.current) {
				const rect = containerRef.current.getBoundingClientRect();
				setViewportSize({ width: rect.width, height: rect.height });
			}
		};

		updateSize();
		window.addEventListener("resize", updateSize);
		return () => window.removeEventListener("resize", updateSize);
	}, []);

	// Load canvas data into store when canvas changes
	useEffect(() => {
		if (canvas) {
			canvasStore.setNodes(canvas.canvasData.nodes);
			canvasStore.setEdges(canvas.canvasData.edges);
			canvasStore.clearSelection();
			// Reset viewport to center
			canvasStore.setViewport(1, 0, 0);
		}
	}, [canvas?.id]);

	// Auto-save when nodes/edges change (debounced)
	useEffect(() => {
		if (!canvas) return;

		const nodesChanged =
			JSON.stringify(debouncedNodes) !==
			JSON.stringify(canvas.canvasData.nodes);
		const edgesChanged =
			JSON.stringify(debouncedEdges) !==
			JSON.stringify(canvas.canvasData.edges);

		if (nodesChanged || edgesChanged) {
			updateCanvas({
				id: canvas.id,
				canvasData: {
					nodes: debouncedNodes,
					edges: debouncedEdges,
				},
			});
		}
	}, [debouncedNodes, debouncedEdges, canvas, updateCanvas]);

	// Handle canvas selection
	const handleSelectCanvas = (selectedCanvas: Canvas) => {
		setSelectedCanvasId(selectedCanvas.id);
	};

	// Handle keyboard shortcuts for save
	useEffect(() => {
		const handleKeyDown = (e: KeyboardEvent) => {
			if ((e.metaKey || e.ctrlKey) && e.key === "s") {
				e.preventDefault();
			if (canvas) {
				updateCanvas({
					id: canvas.id,
					canvasData: {
						nodes: store.nodes,
						edges: store.edges,
					},
				});
			}
			}
		};

		window.addEventListener("keydown", handleKeyDown);
		return () => window.removeEventListener("keydown", handleKeyDown);
	}, [canvas, updateCanvas]);

	return (
		<div className="h-full flex">
			{/* Sidebar with canvas list */}
			<div className="w-64 border-r border-neutral-200 flex-shrink-0">
				<CanvasList
					selectedCanvasId={selectedCanvasId}
					onSelectCanvas={handleSelectCanvas}
				/>
			</div>

			{/* Main canvas area */}
			<div className="flex-1 flex flex-col">
				<CanvasHeader canvas={canvas || null} />
				<CanvasToolbar />

				{/* Canvas viewport */}
				<div
					ref={containerRef}
					className="flex-1 relative bg-neutral-50 overflow-hidden"
				>
					{viewportSize.width > 0 && viewportSize.height > 0 && (
						<CanvasViewport
							width={viewportSize.width}
							height={viewportSize.height}
						/>
					)}
				</div>
			</div>
		</div>
	);
};
