import ky from "ky";

const API_URL = import.meta.env.VITE_API_URL || "http://localhost:9119";

const api = ky.create({
	prefixUrl: API_URL,
	retry: {
		limit: 3,
		methods: ["get", "post", "put", "patch", "delete"],
		statusCodes: [408, 500, 502, 503, 504],
	},
	timeout: 30000,
	hooks: {
		beforeRequest: [
			(request: Request) => {
				request.headers.set("Content-Type", "application/json");
			},
		],
	},
});

export const customFetch = async <T>(
	url: string,
	options?: RequestInit,
): Promise<T> => {
	// Remove leading slash for ky's prefixUrl to work correctly
	const cleanUrl = url.startsWith("/") ? url.slice(1) : url;

	console.log("cleanUrl ->", { cleanUrl, API_URL });

	const response = await api(cleanUrl, {
		method: options?.method as "get" | "post" | "put" | "patch" | "delete",
		body: options?.body as string | undefined,
		headers: options?.headers as Record<string, string>,
	});

	// Handle empty responses
	const contentType = response.headers.get("content-type");
	if (!contentType || !contentType.includes("application/json")) {
		return undefined as T;
	}

	const json = await response.json();

	// Unwrap the response if it has a data property with status/headers wrapper
	if (json && typeof json === "object" && "data" in json && "status" in json) {
		return (json as { data: T }).data;
	}

	return json as T;
};

export default customFetch;
