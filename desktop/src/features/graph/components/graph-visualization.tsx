import { useMemo } from "react";
import { ResourceLink } from "~/aether-sdk/models";

interface GraphNode {
	id: string;
	type: string;
	label: string;
	linkCount: number;
}

interface GraphEdge {
	source: string;
	target: string;
}

interface GraphVisualizationProps {
	links: ResourceLink[];
}

export function GraphVisualization({ links }: GraphVisualizationProps) {
	const { nodes, edges } = useMemo(() => {
		const nodeMap = new Map<string, GraphNode>();
		const edgeSet = new Set<string>();

		// Process links to build nodes and edges
		for (const link of links) {
			// Source node
			const sourceKey = `${link.sourceType}:${link.sourceId}`;
			if (!nodeMap.has(sourceKey)) {
				nodeMap.set(sourceKey, {
					id: sourceKey,
					type: link.sourceType,
					label: `${link.sourceType}:${link.sourceId}`,
					linkCount: 0,
				});
			}
			const sourceNode = nodeMap.get(sourceKey)!;
			sourceNode.linkCount += 1;

			// Target node
			const targetKey = `${link.targetType}:${link.targetId}`;
			if (!nodeMap.has(targetKey)) {
				nodeMap.set(targetKey, {
					id: targetKey,
					type: link.targetType,
					label: `${link.targetType}:${link.targetId}`,
					linkCount: 0,
				});
			}
			const targetNode = nodeMap.get(targetKey)!;
			targetNode.linkCount += 1;

			// Edge
			const edgeKey = `${sourceKey}->${targetKey}`;
			if (!edgeSet.has(edgeKey)) {
				edgeSet.add(edgeKey);
			}
		}

		const nodesArray = Array.from(nodeMap.values());
		const edgesArray = Array.from(edgeSet).map((edgeKey) => {
			const [source, target] = edgeKey.split("->");
			return { source: source!, target: target! };
		});

		return { nodes: nodesArray, edges: edgesArray };
	}, [links]);

	const getNodeColor = (type: string) => {
		switch (type) {
			case "entry":
				return "#3b82f6"; // blue
			case "task":
				return "#10b981"; // green
			case "goal":
				return "#f59e0b"; // amber
			case "canvas":
				return "#8b5cf6"; // purple
			case "bookmark":
				return "#ef4444"; // red
			default:
				return "#6b7280"; // gray
		}
	};

	if (nodes.length === 0) {
		return (
			<div className="h-full flex items-center justify-center">
				<p className="text-sm text-neutral-500">No links found</p>
			</div>
		);
	}

	return (
		<div className="h-full w-full relative">
			<svg
				viewBox={`0 0 ${800} ${600}`}
				className="w-full h-full"
				style={{ background: "#fafafa" }}
			>
				{/* Render edges */}
				{edges.map((edge, idx) => {
					// Simple layout: place nodes in a circle
					const sourceNode = nodes.find((n) => n.id === edge.source);
					const targetNode = nodes.find((n) => n.id === edge.target);
					if (!sourceNode || !targetNode) return null;

					const sourceIdx = nodes.indexOf(sourceNode);
					const targetIdx = nodes.indexOf(targetNode);
					const angle1 = (sourceIdx / nodes.length) * 2 * Math.PI;
					const angle2 = (targetIdx / nodes.length) * 2 * Math.PI;
					const radius = 200;
					const centerX = 400;
					const centerY = 300;
					const x1 = centerX + radius * Math.cos(angle1);
					const y1 = centerY + radius * Math.sin(angle1);
					const x2 = centerX + radius * Math.cos(angle2);
					const y2 = centerY + radius * Math.sin(angle2);

					return (
						<line
							key={`edge-${idx}`}
							x1={x1}
							y1={y1}
							x2={x2}
							y2={y2}
							stroke="#d1d5db"
							strokeWidth={1}
						/>
					);
				})}

				{/* Render nodes */}
				{nodes.map((node, idx) => {
					const angle = (idx / nodes.length) * 2 * Math.PI;
					const radius = 200;
					const centerX = 400;
					const centerY = 300;
					const x = centerX + radius * Math.cos(angle);
					const y = centerY + radius * Math.sin(angle);
					const size = Math.max(20, Math.min(40, 20 + node.linkCount * 2));

					return (
						<g key={node.id}>
							<circle
								cx={x}
								cy={y}
								r={size}
								fill={getNodeColor(node.type)}
								stroke="#fff"
								strokeWidth={2}
							/>
							<text
								x={x}
								y={y + size + 12}
								textAnchor="middle"
								fontSize="10"
								fill="#374151"
							>
								{node.label.length > 15
									? `${node.label.substring(0, 15)}...`
									: node.label}
							</text>
						</g>
					);
				})}
			</svg>
		</div>
	);
}
