import { useState, useEffect } from "react";
import { Save } from "lucide-react";
import { useUpdateCanvas } from "../hooks/use-canvas-api";
import { useSnapshot } from "valtio";
import { canvasStore } from "../canvas.store";
import type { Canvas } from "../types";

interface CanvasHeaderProps {
	canvas: Canvas | null;
}

export function CanvasHeader({ canvas }: CanvasHeaderProps) {
	const [name, setName] = useState(canvas?.name || "");
	const [isEditing, setIsEditing] = useState(false);
	const { mutate: updateCanvas } = useUpdateCanvas();
	const store = useSnapshot(canvasStore);
	const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);

	useEffect(() => {
		if (canvas) {
			setName(canvas.name);
		}
	}, [canvas]);

	const handleNameChange = (newName: string) => {
		setName(newName);
		if (canvas && newName !== canvas.name) {
			setHasUnsavedChanges(true);
		}
	};

	const handleNameBlur = () => {
		setIsEditing(false);
		if (canvas && name !== canvas.name && name.trim()) {
			updateCanvas({ id: canvas.id, name: name.trim() });
			setHasUnsavedChanges(false);
		}
	};

	const handleSave = () => {
		if (canvas) {
			updateCanvas({
				id: canvas.id,
				canvasData: {
					nodes: store.nodes,
					edges: store.edges,
				},
			});
			setHasUnsavedChanges(false);
		}
	};

	if (!canvas) {
		return (
			<div className="h-16 border-b border-neutral-200 flex items-center px-4">
				<h1 className="text-lg font-semibold">Canvas</h1>
			</div>
		);
	}

	return (
		<div className="h-16 border-b border-neutral-200 flex items-center justify-between px-4">
			{isEditing ? (
				<input
					type="text"
					value={name}
					onChange={(e) => handleNameChange(e.target.value)}
					onBlur={handleNameBlur}
					onKeyDown={(e) => {
						if (e.key === "Enter") {
							handleNameBlur();
						}
					}}
					autoFocus
					className="text-lg font-semibold bg-transparent border-none outline-none focus:ring-0"
				/>
			) : (
				<h1
					className="text-lg font-semibold cursor-pointer hover:text-neutral-600"
					onClick={() => setIsEditing(true)}
				>
					{name}
				</h1>
			)}

			<div className="flex items-center gap-2">
				{hasUnsavedChanges && (
					<span className="text-xs text-neutral-500">Unsaved changes</span>
				)}
				<button
					onClick={handleSave}
					className="p-2 hover:bg-neutral-100 rounded-lg transition-colors"
					title="Save canvas (Cmd+S)"
				>
					<Save className="w-4 h-4" />
				</button>
			</div>
		</div>
	);
}
