/**
 * Coordinate transformation utilities for canvas viewport
 */

export function screenToCanvas(
	screenX: number,
	screenY: number,
	zoom: number,
	panX: number,
	panY: number,
): { x: number; y: number } {
	return {
		x: (screenX - panX) / zoom,
		y: (screenY - panY) / zoom,
	};
}

export function canvasToScreen(
	canvasX: number,
	canvasY: number,
	zoom: number,
	panX: number,
	panY: number,
): { x: number; y: number } {
	return {
		x: canvasX * zoom + panX,
		y: canvasY * zoom + panY,
	};
}

export function getNodeConnectionPoint(
	nodeX: number,
	nodeY: number,
	nodeWidth: number,
	nodeHeight: number,
	side: "top" | "right" | "bottom" | "left",
): { x: number; y: number } {
	switch (side) {
		case "top":
			return { x: nodeX + nodeWidth / 2, y: nodeY };
		case "right":
			return { x: nodeX + nodeWidth, y: nodeY + nodeHeight / 2 };
		case "bottom":
			return { x: nodeX + nodeWidth / 2, y: nodeY + nodeHeight };
		case "left":
			return { x: nodeX, y: nodeY + nodeHeight / 2 };
	}
}
