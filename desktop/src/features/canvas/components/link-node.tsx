import { Group, Rect, Text } from "react-konva";
import { useSnapshot } from "valtio";
import { ExternalLink } from "lucide-react";
import { canvasStore } from "../canvas.store";
import type { LinkNode } from "../types";
import { canvasToScreen } from "../utils/coordinates";

interface LinkNodeComponentProps {
	node: LinkNode;
	isSelected: boolean;
}

export function LinkNodeComponent({
	node,
	isSelected,
}: LinkNodeComponentProps) {
	const store = useSnapshot(canvasStore);

	const screenPos = canvasToScreen(
		node.x,
		node.y,
		store.zoom,
		store.panX,
		store.panY,
	);

	const color = node.color
		? `rgba(${node.color.r}, ${node.color.g}, ${node.color.b}, ${node.color.a ?? 1})`
		: "#2563eb";

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
			{/* Link icon placeholder */}
			<Rect
				x={12}
				y={12}
				width={24}
				height={24}
				fill="#dbeafe"
				cornerRadius={2}
			/>
			<Text
				x={44}
				y={16}
				width={node.width - 56}
				text={node.url}
				fontSize={14}
				fontFamily="system-ui, -apple-system, sans-serif"
				fill={color}
			/>
		</Group>
	);
}
