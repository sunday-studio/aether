import { Type, FileText, Link } from "lucide-react";
import { Button } from "react-aria-components";
import { canvasStore } from "../canvas.store";
import type { TextNode, FileNode, LinkNode } from "../types";

export function CanvasToolbar() {
	const generateId = () => `node-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

	const handleAddTextNode = () => {
		const node: TextNode = {
			id: generateId(),
			type: "text",
			x: 100,
			y: 100,
			width: 200,
			height: 100,
			text: "New text node",
		};
		canvasStore.addNode(node);
		canvasStore.selectNode(node.id);
	};

	const handleAddFileNode = () => {
		const node: FileNode = {
			id: generateId(),
			type: "file",
			x: 100,
			y: 100,
			width: 250,
			height: 60,
			file: "",
		};
		canvasStore.addNode(node);
		canvasStore.selectNode(node.id);
	};

	const handleAddLinkNode = () => {
		const node: LinkNode = {
			id: generateId(),
			type: "link",
			x: 100,
			y: 100,
			width: 250,
			height: 60,
			url: "https://example.com",
		};
		canvasStore.addNode(node);
		canvasStore.selectNode(node.id);
	};

	return (
		<div className="flex items-center gap-1 p-2 bg-white border-b border-neutral-200">
			<Button
				onPress={handleAddTextNode}
				className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-neutral-700 hover:bg-neutral-100 rounded-lg transition-colors"
				title="Add text node"
			>
				<Type className="w-4 h-4" />
				Text
			</Button>
			<Button
				onPress={handleAddFileNode}
				className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-neutral-700 hover:bg-neutral-100 rounded-lg transition-colors"
				title="Add file node"
			>
				<FileText className="w-4 h-4" />
				File
			</Button>
			<Button
				onPress={handleAddLinkNode}
				className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-neutral-700 hover:bg-neutral-100 rounded-lg transition-colors"
				title="Add link node"
			>
				<Link className="w-4 h-4" />
				Link
			</Button>
		</div>
	);
}
