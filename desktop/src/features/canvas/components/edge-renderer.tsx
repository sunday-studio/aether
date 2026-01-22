import { Arrow } from "react-konva";
import { useSnapshot } from "valtio";
import { canvasStore } from "../canvas.store";
import { getNodeConnectionPoint, canvasToScreen } from "../utils/coordinates";

export function EdgeRenderer() {
	const store = useSnapshot(canvasStore);

	return (
		<>
			{store.edges.map((edge) => {
				const fromNode = store.nodes.find((n) => n.id === edge.fromNode);
				const toNode = store.nodes.find((n) => n.id === edge.toNode);

				if (!fromNode || !toNode) return null;

				const fromSide = edge.fromSide || "right";
				const toSide = edge.toSide || "left";

				const fromPoint = getNodeConnectionPoint(
					fromNode.x,
					fromNode.y,
					fromNode.width,
					fromNode.height,
					fromSide,
				);

				const toPoint = getNodeConnectionPoint(
					toNode.x,
					toNode.y,
					toNode.width,
					toNode.height,
					toSide,
				);

				const fromScreen = canvasToScreen(
					fromPoint.x,
					fromPoint.y,
					store.zoom,
					store.panX,
					store.panY,
				);

				const toScreen = canvasToScreen(
					toPoint.x,
					toPoint.y,
					store.zoom,
					store.panX,
					store.panY,
				);

				const color = edge.color
					? `rgba(${edge.color.r}, ${edge.color.g}, ${edge.color.b}, ${edge.color.a ?? 1})`
					: "#9ca3af";

				return (
					<Arrow
						key={edge.id}
						points={[fromScreen.x, fromScreen.y, toScreen.x, toScreen.y]}
						stroke={color}
						strokeWidth={2}
						fill={color}
						pointerLength={8}
						pointerWidth={8}
						dashEnabled={false}
					/>
				);
			})}
		</>
	);
}
