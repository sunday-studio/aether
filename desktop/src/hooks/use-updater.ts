import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import type {
	UpdateInfo,
	UpdatePreferences,
	UpdateState,
} from "~/types/updater";

const initialState: UpdateState = {
	checking: false,
	available: false,
	downloading: false,
	progress: 0,
	info: null,
	error: null,
};

/**
 * Hook for managing application updates.
 *
 * Provides functionality to check for updates, download and install them,
 * and manage update preferences.
 */
export function useUpdater() {
	const [state, setState] = useState<UpdateState>(initialState);

	// Check for updates manually
	const checkForUpdates = useCallback(async () => {
		setState((prev) => ({ ...prev, checking: true, error: null }));

		try {
			const info = await invoke<UpdateInfo | null>("check_for_updates");

			if (info) {
				setState((prev) => ({
					...prev,
					checking: false,
					available: true,
					info,
				}));
			} else {
				setState((prev) => ({
					...prev,
					checking: false,
					available: false,
					info: null,
				}));
			}

			return info;
		} catch (error) {
			const message =
				error instanceof Error ? error.message : "Failed to check for updates";
			setState((prev) => ({
				...prev,
				checking: false,
				error: message,
			}));
			return null;
		}
	}, []);

	// Download and install the update
	const downloadAndInstall = useCallback(async () => {
		setState((prev) => ({ ...prev, downloading: true, error: null }));

		try {
			await invoke("download_and_install_update");
			// If we get here, the update was installed and app will restart
		} catch (error) {
			const message =
				error instanceof Error ? error.message : "Failed to install update";
			setState((prev) => ({
				...prev,
				downloading: false,
				error: message,
			}));
		}
	}, []);

	// Skip a specific version
	const skipVersion = useCallback(async (version: string) => {
		try {
			await invoke("skip_update_version", { version });
			setState((prev) => ({
				...prev,
				available: false,
				info: null,
			}));
		} catch (error) {
			console.error("Failed to skip version:", error);
		}
	}, []);

	// Dismiss the update notification temporarily
	const dismissUpdate = useCallback(() => {
		setState((prev) => ({
			...prev,
			available: false,
		}));
	}, []);

	// Get update preferences
	const getPreferences = useCallback(async () => {
		try {
			return await invoke<UpdatePreferences>("get_update_preferences");
		} catch (error) {
			console.error("Failed to get update preferences:", error);
			return null;
		}
	}, []);

	// Set update preferences
	const setPreferences = useCallback(async (preferences: UpdatePreferences) => {
		try {
			await invoke("set_update_preferences", { preferences });
		} catch (error) {
			console.error("Failed to set update preferences:", error);
		}
	}, []);

	// Get current app version
	const getAppVersion = useCallback(async () => {
		try {
			return await invoke<string>("get_app_version");
		} catch (error) {
			console.error("Failed to get app version:", error);
			return null;
		}
	}, []);

	// Listen for update events from backend
	useEffect(() => {
		const unlistenAvailable = listen<UpdateInfo>("update-available", (event) => {
			setState((prev) => ({
				...prev,
				available: true,
				info: event.payload,
			}));
		});

		return () => {
			unlistenAvailable.then((fn) => fn());
		};
	}, []);

	return {
		...state,
		checkForUpdates,
		downloadAndInstall,
		skipVersion,
		dismissUpdate,
		getPreferences,
		setPreferences,
		getAppVersion,
	};
}
