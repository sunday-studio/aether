/**
 * JSON Canvas 1.0 Type Definitions
 * Based on the JSON Canvas specification
 */

export type NodeType = "text" | "file" | "link";

export type EdgeSide = "top" | "right" | "bottom" | "left";

export interface Point {
	x: number;
	y: number;
}

export interface Size {
	width: number;
	height: number;
}

export interface Color {
	r: number;
	g: number;
	b: number;
	a?: number;
}

export interface TextNode {
	id: string;
	type: "text";
	x: number;
	y: number;
	width: number;
	height: number;
	color?: Color;
	text: string;
}

export interface FileNode {
	id: string;
	type: "file";
	x: number;
	y: number;
	width: number;
	height: number;
	file: string;
	subpath?: string;
	color?: Color;
}

export interface LinkNode {
	id: string;
	type: "link";
	x: number;
	y: number;
	width: number;
	height: number;
	url: string;
	color?: Color;
}

export type CanvasNode = TextNode | FileNode | LinkNode;

export interface Edge {
	id: string;
	fromNode: string;
	fromSide?: EdgeSide;
	fromEnd?: "none" | "arrow";
	toNode: string;
	toSide?: EdgeSide;
	toEnd?: "none" | "arrow";
	color?: Color;
	label?: string;
}

export interface CanvasData {
	nodes: CanvasNode[];
	edges: Edge[];
}

export interface Canvas {
	id: string;
	name: string;
	canvasData: CanvasData;
	createdAt: string;
	updatedAt: string;
	deletedAt?: string | null;
}

export interface CanvasState {
	nodes: CanvasNode[];
	edges: Edge[];
	zoom: number;
	panX: number;
	panY: number;
	selectedNodeIds: Set<string>;
	isDragging: boolean;
	dragStart: Point | null;
}
