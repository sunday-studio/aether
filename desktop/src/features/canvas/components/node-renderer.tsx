import { Group } from "react-konva";
import { useSnapshot } from "valtio";
import { canvasStore } from "../canvas.store";
import type { CanvasNode } from "../types";
import { TextNodeComponent } from "./text-node";
import { FileNodeComponent } from "./file-node";
import { LinkNodeComponent } from "./link-node";

interface NodeRendererProps {
	node: CanvasNode;
}

export function NodeRenderer({ node }: NodeRendererProps) {
	const store = useSnapshot(canvasStore);
	const isSelected = store.selectedNodeIds.has(node.id);

	switch (node.type) {
		case "text":
			return <TextNodeComponent node={node} isSelected={isSelected} />;
		case "file":
			return <FileNodeComponent node={node} isSelected={isSelected} />;
		case "link":
			return <LinkNodeComponent node={node} isSelected={isSelected} />;
		default:
			return null;
	}
}
