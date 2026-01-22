import { useEffect, useRef, useState } from "react";
import { Stage, Layer } from "react-konva";
import { useSnapshot } from "valtio";
import { canvasStore } from "../canvas.store";
import { NodeRenderer } from "./node-renderer";
import { EdgeRenderer } from "./edge-renderer";
import { SelectionBox } from "./selection-box";
import { screenToCanvas } from "../utils/coordinates";

interface CanvasViewportProps {
	width: number;
	height: number;
}

export function CanvasViewport({ width, height }: CanvasViewportProps) {
	const stageRef = useRef<any>(null);
	const [isPanning, setIsPanning] = useState(false);
	const [panStart, setPanStart] = useState<{ x: number; y: number } | null>(
		null,
	);
	const [isSpacePressed, setIsSpacePressed] = useState(false);

	const store = useSnapshot(canvasStore);

	// Handle keyboard shortcuts
	useEffect(() => {
		const handleKeyDown = (e: KeyboardEvent) => {
			if (e.code === "Space" && !e.repeat) {
				setIsSpacePressed(true);
			}
			// Undo/Redo
			if ((e.metaKey || e.ctrlKey) && e.key === "z" && !e.shiftKey) {
				e.preventDefault();
				canvasStore.undo();
			}
			if ((e.metaKey || e.ctrlKey) && (e.key === "y" || (e.key === "z" && e.shiftKey))) {
				e.preventDefault();
				canvasStore.redo();
			}
		};

		const handleKeyUp = (e: KeyboardEvent) => {
			if (e.code === "Space") {
				setIsSpacePressed(false);
			}
		};

		window.addEventListener("keydown", handleKeyDown);
		window.addEventListener("keyup", handleKeyUp);

		return () => {
			window.removeEventListener("keydown", handleKeyDown);
			window.removeEventListener("keyup", handleKeyUp);
		};
	}, []);

	// Handle wheel zoom
	const handleWheel = (e: any) => {
		e.evt.preventDefault();

		const stage = e.target.getStage();
		const pointer = stage.getPointerPosition();

		const scaleBy = 1.1;
		const oldZoom = store.zoom;
		const newZoom =
			e.evt.deltaY > 0 ? oldZoom / scaleBy : oldZoom * scaleBy;
		const clampedZoom = Math.max(0.1, Math.min(5, newZoom));

		canvasStore.zoomTo(clampedZoom, pointer.x, pointer.y);
	};

	// Handle pan start
	const handleMouseDown = (e: any) => {
		if (e.evt.button === 1 || (e.evt.button === 0 && isSpacePressed)) {
			// Middle mouse or Space + left mouse
			e.evt.preventDefault();
			setIsPanning(true);
			setPanStart({ x: e.evt.clientX, y: e.evt.clientY });
		} else if (e.evt.button === 0) {
			// Left mouse click
			const stage = e.target.getStage();
			const pointer = stage.getPointerPosition();
			const canvasPos = screenToCanvas(
				pointer.x,
				pointer.y,
				store.zoom,
				store.panX,
				store.panY,
			);

			// Check if clicking on a node
			const clickedNode = store.nodes.find(
				(node) =>
					canvasPos.x >= node.x &&
					canvasPos.x <= node.x + node.width &&
					canvasPos.y >= node.y &&
					canvasPos.y <= node.y + node.height,
			);

			if (clickedNode) {
				canvasStore.selectNode(
					clickedNode.id,
					e.evt.shiftKey || e.evt.metaKey || e.evt.ctrlKey,
				);
				canvasStore.startDrag(pointer.x, pointer.y);
			} else {
				canvasStore.clearSelection();
			}
		}
	};

	// Handle pan/drag
	const handleMouseMove = (e: any) => {
		if (isPanning && panStart) {
			const deltaX = e.evt.clientX - panStart.x;
			const deltaY = e.evt.clientY - panStart.y;
			canvasStore.pan(deltaX, deltaY);
			setPanStart({ x: e.evt.clientX, y: e.evt.clientY });
		} else if (store.isDragging) {
			const stage = e.target.getStage();
			const pointer = stage.getPointerPosition();
			canvasStore.updateDrag(pointer.x, pointer.y);
		}
	};

	// Handle pan/drag end
	const handleMouseUp = () => {
		if (isPanning) {
			setIsPanning(false);
			setPanStart(null);
		}
		canvasStore.endDrag();
	};

	// Prevent context menu
	const handleContextMenu = (e: any) => {
		e.evt.preventDefault();
	};

	return (
		<Stage
			ref={stageRef}
			width={width}
			height={height}
			onWheel={handleWheel}
			onMouseDown={handleMouseDown}
			onMouseMove={handleMouseMove}
			onMouseUp={handleMouseUp}
			onContextMenu={handleContextMenu}
		>
			<Layer>
				{/* Render edges first (behind nodes) */}
				<EdgeRenderer />

				{/* Render nodes */}
				{store.nodes.map((node) => (
					<NodeRenderer key={node.id} node={node} />
				))}

				{/* Selection box */}
				<SelectionBox />
			</Layer>
		</Stage>
	);
}
