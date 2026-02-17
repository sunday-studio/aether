import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { customFetch } from "~/utils/api-client";
import type { Canvas, CanvasData } from "../types";

// Backend response type (snake_case)
interface BackendCanvas {
	id: string;
	name: string;
	canvas_data: unknown; // JSON Canvas format
	created_at: string;
	updated_at: string;
	deleted_at?: string | null;
}

// API response types
interface CanvasResponse {
	data: BackendCanvas;
	status: number;
	headers: Headers;
}

interface CanvasesResponse {
	data: BackendCanvas[];
	status: number;
	headers: Headers;
}

// Convert backend format to frontend format
function convertBackendCanvas(backend: BackendCanvas): Canvas {
	return {
		id: backend.id,
		name: backend.name,
		canvasData: (backend.canvas_data as CanvasData) || { nodes: [], edges: [] },
		createdAt: backend.created_at,
		updatedAt: backend.updated_at,
		deletedAt: backend.deleted_at,
	};
}

// Fetch all canvases
async function fetchCanvases(): Promise<Canvas[]> {
	const response = await customFetch<CanvasesResponse>("GET /v1/canvas", {
		method: "GET",
	});
	return response.data.map(convertBackendCanvas);
}

// Fetch canvas by ID
async function fetchCanvasById(id: string): Promise<Canvas> {
	const response = await customFetch<CanvasResponse>(`GET /v1/canvas/${id}`, {
		method: "GET",
	});
	return convertBackendCanvas(response.data);
}

// Create canvas
async function createCanvas(
	name: string,
	canvasData?: CanvasData,
): Promise<Canvas> {
	const response = await customFetch<CanvasResponse>("POST /v1/canvas", {
		method: "POST",
		body: JSON.stringify({
			name,
			canvas_data: canvasData || { nodes: [], edges: [] },
		}),
	});
	return convertBackendCanvas(response.data);
}

// Update canvas
async function updateCanvas(
	id: string,
	updates: { name?: string; canvasData?: CanvasData },
): Promise<Canvas> {
	const body: { name?: string; canvas_data?: unknown } = {};
	if (updates.name !== undefined) body.name = updates.name;
	if (updates.canvasData !== undefined) body.canvas_data = updates.canvasData;

	const response = await customFetch<CanvasResponse>(`PUT /v1/canvas/${id}`, {
		method: "PUT",
		body: JSON.stringify(body),
	});
	return convertBackendCanvas(response.data);
}

// Delete canvas
async function deleteCanvas(id: string): Promise<void> {
	await customFetch(`DELETE /v1/canvas/${id}`, {
		method: "DELETE",
	});
}

// React Query hooks
export function useCanvases() {
	return useQuery({
		queryKey: ["canvases"],
		queryFn: fetchCanvases,
	});
}

export function useCanvas(id: string | null) {
	return useQuery({
		queryKey: ["canvas", id],
		queryFn: () => (id ? fetchCanvasById(id) : null),
		enabled: !!id,
	});
}

export function useCreateCanvas() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: ({
			name,
			canvasData,
		}: {
			name: string;
			canvasData?: CanvasData;
		}) => createCanvas(name, canvasData),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["canvases"] });
		},
	});
}

export function useUpdateCanvas() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: ({
			id,
			name,
			canvasData,
		}: {
			id: string;
			name?: string;
			canvasData?: CanvasData;
		}) => updateCanvas(id, { name, canvasData }),
		onSuccess: (_, variables) => {
			queryClient.invalidateQueries({ queryKey: ["canvases"] });
			queryClient.invalidateQueries({ queryKey: ["canvas", variables.id] });
		},
	});
}

export function useDeleteCanvas() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: (id: string) => deleteCanvas(id),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["canvases"] });
		},
	});
}
