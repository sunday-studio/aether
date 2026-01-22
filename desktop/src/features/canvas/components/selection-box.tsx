import { Rect } from "react-konva";
import { useSnapshot } from "valtio";
import { canvasStore } from "../canvas.store";
import { canvasToScreen } from "../utils/coordinates";

export function SelectionBox() {
	const store = useSnapshot(canvasStore);

	if (store.selectedNodeIds.size === 0) return null;

	// Calculate bounding box for all selected nodes
	let minX = Infinity;
	let minY = Infinity;
	let maxX = -Infinity;
	let maxY = -Infinity;

	store.selectedNodeIds.forEach((nodeId) => {
		const node = store.nodes.find((n) => n.id === nodeId);
		if (node) {
			minX = Math.min(minX, node.x);
			minY = Math.min(minY, node.y);
			maxX = Math.max(maxX, node.x + node.width);
			maxY = Math.max(maxY, node.y + node.height);
		}
	});

	if (minX === Infinity) return null;

	const padding = 4;
	const boxX = minX - padding;
	const boxY = minY - padding;
	const boxWidth = maxX - minX + padding * 2;
	const boxHeight = maxY - minY + padding * 2;

	const screenPos = canvasToScreen(
		boxX,
		boxY,
		store.zoom,
		store.panX,
		store.panY,
	);

	return (
		<Rect
			x={screenPos.x}
			y={screenPos.y}
			width={boxWidth * store.zoom}
			height={boxHeight * store.zoom}
			stroke="#3b82f6"
			strokeWidth={2}
			dash={[5, 5]}
			fill="rgba(59, 130, 246, 0.1)"
		/>
	);
}
