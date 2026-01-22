import { invoke } from "@tauri-apps/api/core";

// Route to command mapping
const routeToCommand: Record<string, string> = {
	// Tags
	"GET /v1/tags": "get_all_tags",
	"POST /v1/tags": "create_tag",
	"POST /v1/tags/bulk-create": "bulk_create_tags",
	// Entries
	"GET /v1/entry": "get_entries",
	"POST /v1/entry": "create_entry",
	"POST /v1/entry/bulk-create": "bulk_create_entries",
	"GET /v1/entry/:id": "get_entry_by_id",
	"PUT /v1/entry/:id": "update_entry",
	"DELETE /v1/entry/:id": "delete_entry",
	"POST /v1/entry/:id/tags": "add_tags_to_entry",
	"DELETE /v1/entry/:id/tags": "remove_tags_from_entry",
	// Tasks
	"POST /v1/tasks": "create_task",
	"GET /v1/tasks/inbox": "get_inbox_tasks",
	"GET /v1/tasks/overdue": "get_overdue_tasks",
	"GET /v1/tasks/:id": "get_task_by_id",
	"PUT /v1/tasks/:id": "update_task",
	"DELETE /v1/tasks/:id": "delete_task",
	"GET /v1/tasks/:taskId/subtasks": "get_subtasks",
	"POST /v1/tasks/:taskId/subtasks": "create_subtask",
	"PUT /v1/tasks/:taskId/subtasks/:subtaskId": "update_subtask",
	"DELETE /v1/tasks/:taskId/subtasks/:subtaskId": "delete_subtask",
	"POST /v1/tasks/:taskId/subtasks/reorder": "reorder_subtasks",
	"POST /v1/tasks/:id/tags": "add_tags_to_task",
	"DELETE /v1/tasks/:id/tags": "remove_tags_from_task",
	"POST /v1/tasks/:id/goal": "add_goal_to_task",
	"DELETE /v1/tasks/:id/goal": "remove_goal_from_task",
	// Goals
	"GET /v1/goals": "get_goals",
	"POST /v1/goals": "create_goal",
	"GET /v1/goals/:id": "get_goal_by_id",
	"PUT /v1/goals/:id": "update_goal",
	"DELETE /v1/goals/:id": "delete_goal",
	"GET /v1/goals/:goalId/instances": "get_goal_instances",
	"GET /v1/goals/:goalId/instances/current": "get_current_goal_instance",
	"POST /v1/goals/:id/tags": "add_tags_to_goal",
	"DELETE /v1/goals/:id/tags": "remove_tags_from_goal",
	// Trash
	"GET /v1/trash/tasks": "get_trashed_tasks",
	"POST /v1/trash/:id/restore": "restore_task",
	// Sync
	"POST /v1/sync/configure": "configure_sync",
	"POST /v1/sync": "sync",
	// Activities
	"GET /v1/activities": "get_activities",
	// Search
	"GET /v1/search": "search_resources",
	// Bookmarks
	"GET /v1/bookmarks": "get_bookmarks",
	"POST /v1/bookmarks": "create_bookmark",
	"GET /v1/bookmarks/:id": "get_bookmark_by_id",
	"PUT /v1/bookmarks/:id": "update_bookmark",
	"DELETE /v1/bookmarks/:id": "delete_bookmark",
	"POST /v1/bookmarks/:id/tags": "add_tags_to_bookmark",
	"DELETE /v1/bookmarks/:id/tags": "remove_tags_from_bookmark",
	"GET /v1/bookmarks/extract-metadata": "extract_metadata",
	// Links
	"POST /v1/links": "create_link",
	"GET /v1/links/backlinks": "get_backlinks",
	"GET /v1/links/outgoing": "get_outgoing_links",
	"DELETE /v1/links": "delete_link",
	"GET /v1/links/search": "search_linkable_resources",
	"GET /v1/links/graph": "get_all_links_for_graph",
	"POST /v1/links/sync": "sync_links_from_content",
	// Audio
	"POST /v1/audio": "save_audio_recording",
	"GET /v1/audio/:mediaId": "get_audio_data",
	"DELETE /v1/audio/:mediaId": "delete_audio_recording",
	"GET /v1/entry/:entryId/media": "get_media_items_for_entry",
	"GET /v1/audio/:mediaId/metadata": "get_audio_metadata",
	// Transcription
	"POST /v1/transcription": "start_transcription",
	"GET /v1/transcription/:mediaId": "get_transcriptions",
	"GET /v1/transcription/by-id/:transcriptionId": "get_transcription_by_id",
	"POST /v1/transcription/set-active": "set_active_transcription",
	"GET /v1/transcription/providers": "list_providers",
	"POST /v1/transcription/validate-provider": "validate_provider",
	"GET /v1/transcription/models": "list_available_models",
	"POST /v1/transcription/models/download": "download_model",
	"POST /v1/transcription/models/verify": "verify_model",
	"DELETE /v1/transcription/models/:modelSize": "delete_model",
	// Settings
	"GET /v1/settings": "get_setting",
	"POST /v1/settings": "set_setting",
	// Canvas
	"GET /v1/canvas": "get_canvases",
	"GET /v1/canvas/:id": "get_canvas_by_id",
	"POST /v1/canvas": "create_canvas",
	"PUT /v1/canvas/:id": "update_canvas",
	"DELETE /v1/canvas/:id": "delete_canvas",
};

// Extract path parameters from URL
function extractPathParams(
	routePattern: string,
	url: string,
): Record<string, string> {
	const patternParts = routePattern.split("/");
	const urlParts = url.split("/").filter((p) => p);

	const params: Record<string, string> = {};

	for (let i = 0; i < patternParts.length; i++) {
		const patternPart = patternParts[i];
		if (patternPart?.startsWith(":")) {
			const paramName = patternPart.slice(1);
			// Adjust index: patternParts[0] is empty string, urlParts[0] is first real part
			// So patternParts[i] maps to urlParts[i - 1] when i > 0
			const urlIndex = i > 0 ? i - 1 : i;
			params[paramName] = urlParts[urlIndex] || "";
		}
	}

	return params;
}

// Extract query parameters from URL
function extractQueryParams(url: string): Record<string, string> {
	const params: Record<string, string> = {};
	const queryString = url.split("?")[1];
	if (!queryString) return params;

	const pairs = queryString.split("&");
	for (const pair of pairs) {
		const [key, value] = pair.split("=");
		if (key && value) {
			params[decodeURIComponent(key)] = decodeURIComponent(value);
		}
	}
	return params;
}

// Find matching route pattern
function findMatchingRoute(
	method: string,
	url: string,
): {
	command: string;
	params: Record<string, string>;
	queryParams: Record<string, string>;
} | null {
	// Remove query string and normalize URL
	const cleanUrl = url.split("?")[0];
	const urlPath = cleanUrl.startsWith("/") ? cleanUrl : `/${cleanUrl}`;
	const queryParams = extractQueryParams(url);

	// Try exact match first
	const exactKey = `${method} ${urlPath}`;
	if (routeToCommand[exactKey]) {
		return {
			command: routeToCommand[exactKey],
			params: {},
			queryParams,
		};
	}

	// Try pattern matching with path parameters
	for (const [routePattern, command] of Object.entries(routeToCommand)) {
		const [routeMethod, routePath] = routePattern.split(" ", 2);
		if (routeMethod !== method) continue;

		const routeParts = routePath.split("/");
		const urlParts = urlPath.split("/").filter((p) => p);

		if (routeParts.length !== urlParts.length + 1) continue; // +1 for empty first part

		// Check if pattern matches
		let matches = true;
		for (let i = 1; i < routeParts.length; i++) {
			const routePart = routeParts[i];
			const urlPart = urlParts[i - 1];

			if (routePart?.startsWith(":")) {
				continue; // Parameter, matches anything
			}
			if (routePart !== urlPart) {
				matches = false;
				break;
			}
		}

		if (matches) {
			const params = extractPathParams(routePath, urlPath);
			return { command, params, queryParams };
		}
	}

	return null;
}

export const customFetch = async <T>(
	url: string,
	options?: RequestInit,
): Promise<T> => {
	const method = (options?.method || "GET").toUpperCase();
	const body = options?.body ? JSON.parse(options.body as string) : undefined;

	// Find matching route
	const match = findMatchingRoute(method, url);
	if (!match) {
		throw new Error(`No matching route found for ${method} ${url}`);
	}

	try {
		// Prepare command arguments
		// Start with path params (these take precedence)
		const args: Record<string, unknown> = { ...match.params };
		// Add query parameters (for GET requests with query params)
		// Convert numeric parameters to numbers
		const numericParams = new Set(["limit", "offset"]);
		for (const [key, value] of Object.entries(match.queryParams)) {
			if (!(key in args)) {
				// Convert numeric query parameters to numbers
				if (numericParams.has(key)) {
					const numValue = Number(value);
					// Only set if it's a valid number, otherwise skip (will be undefined/optional)
					if (!Number.isNaN(numValue) && value.trim() !== "") {
						args[key] = numValue;
					}
				} else {
					args[key] = value;
				}
			}
		}
		if (body !== undefined) {
			// If body is an object, merge it into args
			// But don't overwrite path params - they take precedence
			if (typeof body === "object" && !Array.isArray(body) && body !== null) {
				// Merge body into args, but preserve path params
				for (const [key, value] of Object.entries(body)) {
					// Only add if not already set by path params
					if (!(key in args)) {
						args[key] = value;
					}
				}
			} else {
				// For array bodies or other types, use 'payload' key
				args.payload = body;
			}
		}

		// Invoke Tauri command
		const result = await invoke(match.command, args);

		// Wrap response in Orval's expected format
		const wrappedResponse = {
			data: result,
			status: 200,
			headers: new Headers({ "content-type": "application/json" }),
		} as T;

		return wrappedResponse;
	} catch (error) {
		// Handle Tauri errors
		let status = 500;
		let errorData: unknown = error;

		if (error && typeof error === "object" && "message" in error) {
			const message = String(error.message);
			if (message.includes("not found") || message.includes("NotFound")) {
				status = 404;
			} else if (
				message.includes("bad request") ||
				message.includes("BadRequest")
			) {
				status = 400;
			} else if (message.includes("conflict") || message.includes("Conflict")) {
				status = 409;
			}
			errorData = { message };
		}

		const wrappedError = {
			data: errorData,
			status,
			headers: new Headers({ "content-type": "application/json" }),
		} as T;

		throw wrappedError;
	}
};

export default customFetch;
