import { createSyncStoragePersister } from "@tanstack/query-sync-storage-persister";
import { QueryClient } from "@tanstack/react-query";
import { persistQueryClient } from "@tanstack/react-query-persist-client";

// Check if error is a transient/retryable error (like STREAM_EXPIRED from libSQL)
const isRetryableError = (error: unknown): boolean => {
	if (error instanceof Error) {
		const message = error.message.toLowerCase();
		// Retry on stream expiration, network errors, and 5xx server errors
		return (
			message.includes("stream_expired") ||
			message.includes("stream has expired") ||
			message.includes("network") ||
			message.includes("fetch failed") ||
			message.includes("connection") ||
			message.includes("timeout")
		);
	}
	return false;
};

export const initQueryClient = () => {
	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				refetchOnWindowFocus: false,
				gcTime: 1000 * 60 * 60 * 24 * 7, // 7 days

				// Retry transient errors like STREAM_EXPIRED up to 3 times
				retry: (failureCount, error) => {
					if (failureCount >= 3) return false;
					return isRetryableError(error);
				},
				retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 10000),
			},
			mutations: {
				// Also retry mutations for transient errors
				retry: (failureCount, error) => {
					if (failureCount >= 2) return false;
					return isRetryableError(error);
				},
				retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 5000),
			},
		},
	});

	const localStoragePersister = createSyncStoragePersister({
		storage: window.localStorage,
	});

	persistQueryClient({
		queryClient,
		dehydrateOptions: {
			shouldDehydrateQuery(query) {
				if (query.meta?.persist === false) return false;
				return query.state.status === "success";
			},
		},
		persister: localStoragePersister,
	});

	return queryClient;
};
