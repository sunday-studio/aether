export type ProviderChoice = 'openai' | 'groq';
export type StepId = 'intro' | 'profile' | 'sync' | 'ai';
export type SyncChoice = 'yes' | 'no' | null;
export type AiChoice = 'yes' | 'no' | null;

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
