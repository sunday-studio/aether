import { useState } from "react";
import { Plus, Trash2 } from "lucide-react";
import { Button } from "react-aria-components";
import { useCanvases, useCreateCanvas, useDeleteCanvas } from "../hooks/use-canvas-api";
import { cn } from "~/utils/cn";
import type { Canvas } from "../types";

interface CanvasListProps {
	selectedCanvasId: string | null;
	onSelectCanvas: (canvas: Canvas) => void;
}

export function CanvasList({
	selectedCanvasId,
	onSelectCanvas,
}: CanvasListProps) {
	const { data: canvases, isLoading } = useCanvases();
	const { mutate: createCanvas } = useCreateCanvas();
	const { mutate: deleteCanvas } = useDeleteCanvas();

	const handleCreateCanvas = () => {
		createCanvas({
			name: "New Canvas",
			canvasData: { nodes: [], edges: [] },
		});
	};

	const handleDeleteCanvas = (e: React.MouseEvent, canvasId: string) => {
		e.stopPropagation();
		if (confirm("Are you sure you want to delete this canvas?")) {
			deleteCanvas(canvasId);
		}
	};

	if (isLoading) {
		return (
			<div className="p-4 text-sm text-neutral-500">Loading canvases...</div>
		);
	}

	return (
		<div className="h-full flex flex-col">
			<div className="p-4 border-b border-neutral-200">
				<Button
					onPress={handleCreateCanvas}
					className="w-full flex items-center gap-2 px-3 py-2 text-sm font-medium text-neutral-700 bg-neutral-100 hover:bg-neutral-200 rounded-lg transition-colors"
				>
					<Plus className="w-4 h-4" />
					New Canvas
				</Button>
			</div>

			<div className="flex-1 overflow-y-auto">
				{canvases && canvases.length > 0 ? (
					<div className="p-2">
						{canvases.map((canvas) => (
							<div
								key={canvas.id}
								onClick={() => onSelectCanvas(canvas)}
								className={cn(
									"group relative p-3 mb-1 rounded-lg cursor-pointer transition-colors",
									selectedCanvasId === canvas.id
										? "bg-neutral-900 text-white"
										: "hover:bg-neutral-100",
								)}
							>
								<div className="font-medium text-sm truncate">
									{canvas.name}
								</div>
								<button
									onClick={(e) => handleDeleteCanvas(e, canvas.id)}
									className={cn(
										"absolute right-2 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 p-1 rounded transition-opacity",
										selectedCanvasId === canvas.id
											? "text-white hover:bg-white/20"
											: "text-neutral-500 hover:bg-neutral-200",
									)}
								>
									<Trash2 className="w-4 h-4" />
								</button>
							</div>
						))}
					</div>
				) : (
					<div className="p-4 text-sm text-neutral-500 text-center">
						No canvases yet. Create one to get started!
					</div>
				)}
			</div>
		</div>
	);
}
