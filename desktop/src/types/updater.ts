/** Information about an available update */
export interface UpdateInfo {
	currentVersion: string;
	latestVersion: string;
	changelog: string;
	publishedAt: string | null;
}

/** User preferences for update behavior */
export interface UpdatePreferences {
	autoCheck: boolean;
	autoDownload: boolean;
	skippedVersions: string[];
}

/** Progress during update download */
export interface UpdateProgress {
	downloadedBytes: number;
	totalBytes: number;
	percentage: number;
}

/** Current state of the update system */
export interface UpdateState {
	checking: boolean;
	available: boolean;
	downloading: boolean;
	progress: number;
	info: UpdateInfo | null;
	error: string | null;
	lastSuccessfulCheck: string | null;
}

/** Persisted status of the last successful update-feed request. */
export interface UpdateCheckStatus {
	lastSuccessfulCheck: string | null;
}

/** The outcome of a manually requested update check. */
export type UpdateCheckResult =
	| { succeeded: true; info: UpdateInfo | null }
	| { succeeded: false; info: null };
