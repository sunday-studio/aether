import { Group, Rect, Text } from "react-konva";
import { useSnapshot } from "valtio";
import { canvasStore } from "../canvas.store";
import type { TextNode } from "../types";
import { canvasToScreen } from "../utils/coordinates";

interface TextNodeComponentProps {
	node: TextNode;
	isSelected: boolean;
}

export function TextNodeComponent({
	node,
	isSelected,
}: TextNodeComponentProps) {
	const store = useSnapshot(canvasStore);

	const screenPos = canvasToScreen(
		node.x,
		node.y,
		store.zoom,
		store.panX,
		store.panY,
	);

	const screenWidth = node.width * store.zoom;
	const screenHeight = node.height * store.zoom;

	const color = node.color
		? `rgba(${node.color.r}, ${node.color.g}, ${node.color.b}, ${node.color.a ?? 1})`
		: "#000000";

	return (
		<Group x={screenPos.x} y={screenPos.y} scaleX={store.zoom} scaleY={store.zoom}>
			<Rect
				width={node.width}
				height={node.height}
				fill="#ffffff"
				stroke={isSelected ? "#3b82f6" : "#e5e7eb"}
				strokeWidth={isSelected ? 2 : 1}
				cornerRadius={4}
			/>
			<Text
				x={8}
				y={8}
				width={node.width - 16}
				height={node.height - 16}
				text={node.text}
				fontSize={14}
				fontFamily="system-ui, -apple-system, sans-serif"
				fill={color}
				wrap="word"
			/>
		</Group>
	);
}
