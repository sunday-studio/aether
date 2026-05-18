import { customFetch } from '~/lib/api-client';

export const DEFAULT_SEARCH_EMBEDDING_MODEL = 'all-MiniLM-L6-v2';

export interface SearchEmbeddingModel {
	name: string;
	size: string;
	dimensions: number | null;
	fileSize: number;
	downloadUrl: string | null;
	isDownloaded: boolean;
	modelPath: string | null;
	modelsDirectory: string | null;
}

export interface SearchEmbeddingStatus {
	totalEmbeddings: number;
	models: Array<{
		modelName: string;
		dimensions: number;
		totalEmbeddings: number;
	}>;
}

export interface SearchIndexStatus {
	totalDocuments: number;
	resourceCounts: Record<string, number>;
}

interface ApiResponse<T> {
	data: T;
	status: number;
}

export async function listSearchEmbeddingModels() {
	const response = await customFetch<ApiResponse<SearchEmbeddingModel[]>>(
		'/v1/search/embedding-models',
		{ method: 'GET' },
	);
	return response.data;
}

export async function downloadSearchEmbeddingModel(modelName = DEFAULT_SEARCH_EMBEDDING_MODEL) {
	const response = await customFetch<ApiResponse<string>>(
		`/v1/search/embedding-models/${encodeURIComponent(modelName)}/download`,
		{ method: 'POST' },
	);
	return response.data;
}

export async function reindexSearchDocuments() {
	const response = await customFetch<ApiResponse<SearchIndexStatus>>('/v1/search/index/reindex', {
		method: 'POST',
	});
	return response.data;
}

export async function indexSearchEmbeddings(modelName = DEFAULT_SEARCH_EMBEDDING_MODEL) {
	const response = await customFetch<ApiResponse<SearchEmbeddingStatus>>(
		'/v1/search/embeddings/index',
		{
			method: 'POST',
			body: JSON.stringify({ modelName }),
		},
	);
	return response.data;
}

export function formatModelSize(bytes: number) {
	if (!Number.isFinite(bytes) || bytes <= 0) return 'Unknown size';
	const megabytes = bytes / 1_000_000;
	return `${Math.round(megabytes)} MB`;
}
