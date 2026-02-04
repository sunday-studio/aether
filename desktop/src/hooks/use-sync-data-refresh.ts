// @refresh reset
import { useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import type { SyncStatus } from "~/aether-sdk/models";

// Global flag to prevent multiple registrations
const syncListenerRegistered = false;
const globalUnlisten: (() => void) | null = null;

/**
 * Hook that listens for sync events and invalidates all data queries.
 *
 * When sync pulls changes from the server and writes them to the local database,
 * TanStack Query's cache becomes stale. This hook invalidates all queries
 * (except sync-related ones) to ensure the UI reflects the latest data.
 *
 * Since we're querying a local SQLite database, the refetch cost is minimal (~1-10ms).
 */
export function useSyncDataRefresh() {
	const queryClient = useQueryClient();

	// useEffect(() => {
	// 	// If already registered globally, don't register again
	// 	if (syncListenerRegistered) {
	// 		return;
	// 	}

	// 	syncListenerRegistered = true;

	// 	// Async setup
	// 	(async () => {
	// 		try {
	// 			const unlistenFn = await listen<SyncStatus>("sync-status", () => {
	// 				// Invalidate all queries except sync-related ones
	// 				queryClient.invalidateQueries({
	// 					predicate: (query) => {
	// 						const key = query.queryKey;
	// 						// Skip sync-related queries - they're already handled by the sync section
	// 						if (Array.isArray(key) && key[0] === "sync") return false;
	// 						return true;
	// 					},
	// 				});
	// 			});

	// 			globalUnlisten = unlistenFn;
	// 		} catch (error) {
	// 			console.error("Failed to setup sync listener:", error);
	// 			syncListenerRegistered = false;
	// 		}
	// 	})();

	// 	// Cleanup only on full unmount (not on hot reload)
	// 	return () => {
	// 		// Don't cleanup during development hot reload
	// 		if (import.meta.env.DEV && import.meta.hot) {
	// 			return;
	// 		}

	// 		if (globalUnlisten) {
	// 			globalUnlisten();
	// 			globalUnlisten = null;
	// 		}
	// 		syncListenerRegistered = false;
	// 	};
	// }, [queryClient]);
}
