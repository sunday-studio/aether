export type StepId = 'intro' | 'profile' | 'sync' | 'search';
export type SyncChoice = 'yes' | 'no' | null;
export type SearchChoice = 'yes' | 'no' | null;

export interface PreviewItem {
	label: string;
	value: string;
	active: boolean;
}

export interface EmbeddingModelSummary {
	name: string;
	fileSize: number;
	isDownloaded: boolean;
	modelPath?: string | null;
}
