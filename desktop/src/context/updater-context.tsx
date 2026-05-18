import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { createContext, useCallback, useContext, useEffect, useMemo, useState } from 'react';
import type { UpdateInfo, UpdatePreferences, UpdateProgress, UpdateState } from '~/types/updater';

const initialState: UpdateState = {
	checking: false,
	available: false,
	downloading: false,
	progress: 0,
	info: null,
	error: null,
};

interface UpdaterContextValue extends UpdateState {
	checkForUpdates: () => Promise<UpdateInfo | null>;
	downloadAndInstall: () => Promise<void>;
	skipVersion: (version: string) => Promise<void>;
	dismissUpdate: () => void;
	getPreferences: () => Promise<UpdatePreferences | null>;
	setPreferences: (preferences: UpdatePreferences) => Promise<void>;
	getAppVersion: () => Promise<string | null>;
}

const UpdaterContext = createContext<UpdaterContextValue | null>(null);

export function UpdaterProvider({ children }: { children: React.ReactNode }) {
	const [state, setState] = useState<UpdateState>(initialState);

	const checkForUpdates = useCallback(async () => {
		setState(prev => ({ ...prev, checking: true, error: null }));

		try {
			const info = await invoke<UpdateInfo | null>('check_for_updates');

			setState(prev => ({
				...prev,
				checking: false,
				available: Boolean(info),
				info,
			}));

			return info;
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Failed to check for updates';
			setState(prev => ({
				...prev,
				checking: false,
				error: message,
			}));
			return null;
		}
	}, []);

	const downloadAndInstall = useCallback(async () => {
		setState(prev => ({ ...prev, downloading: true, error: null }));

		try {
			await invoke('download_and_install_update');
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Failed to install update';
			setState(prev => ({
				...prev,
				downloading: false,
				error: message,
			}));
		}
	}, []);

	const skipVersion = useCallback(async (version: string) => {
		try {
			await invoke('skip_update_version', { version });
			setState(prev => ({
				...prev,
				available: false,
				info: null,
			}));
		} catch (error) {
			console.error('Failed to skip version:', error);
		}
	}, []);

	const dismissUpdate = useCallback(() => {
		setState(prev => ({
			...prev,
			available: false,
		}));
	}, []);

	const getPreferences = useCallback(async () => {
		try {
			return await invoke<UpdatePreferences>('get_update_preferences');
		} catch (error) {
			console.error('Failed to get update preferences:', error);
			return null;
		}
	}, []);

	const setPreferences = useCallback(async (preferences: UpdatePreferences) => {
		try {
			await invoke('set_update_preferences', { preferences });
		} catch (error) {
			console.error('Failed to set update preferences:', error);
		}
	}, []);

	const getAppVersion = useCallback(async () => {
		try {
			return await invoke<string>('get_app_version');
		} catch (error) {
			console.error('Failed to get app version:', error);
			return null;
		}
	}, []);

	useEffect(() => {
		const unlistenAvailable = listen<UpdateInfo>('update-available', event => {
			setState(prev => ({
				...prev,
				available: true,
				info: event.payload,
				error: null,
			}));
		});
		const unlistenProgress = listen<UpdateProgress>('update-download-progress', event => {
			setState(prev => ({
				...prev,
				downloading: true,
				progress: event.payload.percentage,
			}));
		});

		return () => {
			unlistenAvailable.then(fn => fn());
			unlistenProgress.then(fn => fn());
		};
	}, []);

	const value = useMemo<UpdaterContextValue>(
		() => ({
			...state,
			checkForUpdates,
			downloadAndInstall,
			skipVersion,
			dismissUpdate,
			getPreferences,
			setPreferences,
			getAppVersion,
		}),
		[
			state,
			checkForUpdates,
			downloadAndInstall,
			skipVersion,
			dismissUpdate,
			getPreferences,
			setPreferences,
			getAppVersion,
		],
	);

	return <UpdaterContext.Provider value={value}>{children}</UpdaterContext.Provider>;
}

export function useUpdaterContext() {
	const context = useContext(UpdaterContext);
	if (!context) {
		throw new Error('useUpdater must be used within UpdaterProvider');
	}
	return context;
}
