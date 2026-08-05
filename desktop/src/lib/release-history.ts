import { invoke } from '@tauri-apps/api/core';

export interface ReleaseHistoryItem {
	tagName: string;
	name: string;
	notes: string;
	publishedAt: string | null;
	url: string;
}

export function getReleaseHistory() {
	return invoke<ReleaseHistoryItem[]>('get_release_history');
}
