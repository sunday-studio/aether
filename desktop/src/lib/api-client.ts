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
			params[paramName] = urlParts[i] || "";
		}
	}

	return params;
}

// Find matching route pattern
function findMatchingRoute(
	method: string,
	url: string,
): { command: string; params: Record<string, string> } | null {
	// Remove query string and normalize URL
	const cleanUrl = url.split("?")[0];
	const urlPath = cleanUrl.startsWith("/") ? cleanUrl : `/${cleanUrl}`;

	// Try exact match first
	const exactKey = `${method} ${urlPath}`;
	if (routeToCommand[exactKey]) {
		return {
			command: routeToCommand[exactKey],
			params: {},
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
			return { command, params };
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
		const args: Record<string, unknown> = { ...match.params };
		if (body !== undefined) {
			// If body is an object, merge it into args
			if (typeof body === "object" && !Array.isArray(body) && body !== null) {
				Object.assign(args, body);
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
