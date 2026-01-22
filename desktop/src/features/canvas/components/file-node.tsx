import { Group, Rect, Text, Image } from "react-konva";
import { useSnapshot } from "valtio";
import { File } from "lucide-react";
import { canvasStore } from "../canvas.store";
import type { FileNode } from "../types";
import { canvasToScreen } from "../utils/coordinates";

interface FileNodeComponentProps {
	node: FileNode;
	isSelected: boolean;
}

export function FileNodeComponent({
	node,
	isSelected,
}: FileNodeComponentProps) {
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
		: "#000000";

	// Extract filename from path
	const filename = node.file.split("/").pop() || node.file;

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
			{/* File icon placeholder */}
			<Rect
				x={12}
				y={12}
				width={24}
				height={24}
				fill="#f3f4f6"
				cornerRadius={2}
			/>
			<Text
				x={44}
				y={16}
				width={node.width - 56}
				text={filename}
				fontSize={14}
				fontFamily="system-ui, -apple-system, sans-serif"
				fill={color}
			/>
			{node.subpath && (
				<Text
					x={44}
					y={32}
					width={node.width - 56}
					text={node.subpath}
					fontSize={12}
					fontFamily="system-ui, -apple-system, sans-serif"
					fill="#6b7280"
				/>
			)}
		</Group>
	);
}
