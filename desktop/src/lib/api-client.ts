import ky from "ky";

const API_URL = import.meta.env.VITE_API_URL || "http://localhost:9119";

console.log("vite url ->", import.meta.env.VITE_API_URL);

const api = ky.create({
	prefixUrl: API_URL,
	retry: {
		limit: 3,
		methods: ["get", "post", "put", "patch", "delete"],
		statusCodes: [408, 500, 502, 503, 504],
	},
	timeout: 30000,
	// Don't throw on HTTP errors - we'll handle them ourselves
	throwHttpErrors: false,
	hooks: {
		beforeRequest: [
			(request: Request) => {
				request.headers.set("Content-Type", "application/json");
			},
		],
		beforeRetry: [
			async ({ error }) => {
				// Don't retry if request was aborted
				if (error instanceof Error && error.name === "AbortError") {
					throw error;
				}
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

	try {
		const response = await api(cleanUrl, {
			method: options?.method as "get" | "post" | "put" | "patch" | "delete",
			body: options?.body as string | undefined,
			headers: options?.headers as Record<string, string>,
			signal: options?.signal,
		});

		// Handle empty responses (204 No Content, etc.)
		const contentType = response.headers.get("content-type");
		if (
			response.status === 204 ||
			!contentType ||
			!contentType.includes("application/json")
		) {
			// Return wrapped empty response for Orval
			return {
				data: undefined,
				status: response.status,
				headers: response.headers,
			} as T;
		}

		const json = await response.json();

		// Wrap ALL responses (success and error) in Orval's expected format
		// Orval generates types as: { data: T, status: number, headers: Headers }
		const wrappedResponse = {
			data: json,
			status: response.status,
			headers: response.headers,
		} as T;

		// For error status codes, throw the wrapped response
		// This allows react-query to catch errors properly
		if (!response.ok) {
			throw wrappedResponse;
		}

		return wrappedResponse;
	} catch (error) {
		// If it's already our wrapped response, re-throw it
		if (
			error &&
			typeof error === "object" &&
			"data" in error &&
			"status" in error
		) {
			throw error;
		}

		// Handle AbortErrors gracefully (expected when requests are cancelled)
		if (error instanceof Error && error.name === "AbortError") {
			// Don't log AbortErrors - they're normal when components unmount
			throw error;
		}

		// For network errors or other issues, log and re-throw
		console.error("Network error:", error);
		throw error;
	}
};

export default customFetch;
