import { useQueryClient } from '@tanstack/react-query';
import { listen } from '@tauri-apps/api/event';
import { Download, Sparkles } from 'lucide-react';
import { useEffect } from 'react';
import { toast } from 'sonner';
import {
	DEFAULT_SEARCH_EMBEDDING_MODEL,
	indexSearchEmbeddings,
	reindexSearchDocuments,
} from '~/lib/search-embedding-models';

interface SearchEmbeddingModelEvent {
	modelName: string;
	progress: number | null;
	modelPath: string | null;
	message: string | null;
}

export function SearchEmbeddingNotificationListener() {
	const queryClient = useQueryClient();

	useEffect(() => {
		const unlistenReady = listen<SearchEmbeddingModelEvent>(
			'search-embedding-model-ready',
			event => {
				const modelName = event.payload.modelName || DEFAULT_SEARCH_EMBEDDING_MODEL;
				void queryClient.invalidateQueries({ queryKey: ['search-embedding-models'] });

				toast('Local search model downloaded', {
					description: 'Aether can now build local semantic search on this device.',
					icon: <Download className='size-4' />,
				});

				window.setTimeout(() => {
					toast('Building local search context', {
						description: 'Aether is preparing local notes and tasks for embeddings.',
						icon: <Sparkles className='size-4' />,
					});

					reindexSearchDocuments()
						.then(() => indexSearchEmbeddings(modelName))
						.then(status => {
							void queryClient.invalidateQueries({ queryKey: ['search-embedding-models'] });
							toast('Local search context is ready', {
								description: `${status.totalEmbeddings} embeddings indexed on this device.`,
								icon: <Sparkles className='size-4' />,
							});
						})
						.catch(error => {
							const message = error instanceof Error ? error.message : String(error);
							toast.error('Embedding index failed', { description: message });
						});
				}, 3500);
			},
		);

		const unlistenFailed = listen<SearchEmbeddingModelEvent>(
			'search-embedding-model-download-failed',
			event => {
				toast.error('Local search model download failed', {
					description: event.payload.message ?? 'Try again from Settings.',
				});
			},
		);

		return () => {
			unlistenReady.then(fn => fn());
			unlistenFailed.then(fn => fn());
		};
	}, [queryClient]);

	return null;
}
