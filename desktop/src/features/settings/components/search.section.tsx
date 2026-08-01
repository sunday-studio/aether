import { useQuery } from '@tanstack/react-query';
import { CheckCircle2, CircleAlert, Download, HardDrive, RefreshCw } from 'lucide-react';
import { useState } from 'react';
import { Button } from '~/components/shared/button';
import {
	DEFAULT_SEARCH_EMBEDDING_MODEL,
	downloadSearchEmbeddingModel,
	formatModelSize,
	indexSearchEmbeddings,
	listSearchEmbeddingModels,
	reindexSearchDocuments,
} from '~/lib/search-embedding-models';
import { useSettingsStore } from '~/store/settings-store';
import { cn } from '~/utils/cn';

const SEARCH_EMBEDDINGS_ENABLED_KEY = 'search.embeddings.enabled';
const SEARCH_EMBEDDINGS_PROVIDER_KEY = 'search.embeddings.provider';
const SEARCH_EMBEDDINGS_MODEL_KEY = 'search.embeddings.model';
const SEARCH_EMBEDDINGS_AUTO_INDEX_KEY = 'search.embeddings.auto_index';

function getErrorMessage(error: unknown) {
	if (error && typeof error === 'object' && 'data' in error) {
		const data = (error as { data?: unknown }).data;
		if (data && typeof data === 'object' && 'message' in data) {
			return String((data as { message?: unknown }).message);
		}
	}
	return error instanceof Error ? error.message : String(error);
}

export const SearchSection = () => {
	const [isStartingModelDownload, setIsStartingModelDownload] = useState(false);
	const [isIndexingEmbeddings, setIsIndexingEmbeddings] = useState(false);
	const [statusMessage, setStatusMessage] = useState<string | null>(null);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);
	const { setValues } = useSettingsStore();
	const { data: embeddingModels, refetch: refetchEmbeddingModels } = useQuery({
		queryKey: ['search-embedding-models'],
		queryFn: listSearchEmbeddingModels,
	});
	const selectedEmbeddingModel =
		embeddingModels?.find(model => model.name === DEFAULT_SEARCH_EMBEDDING_MODEL) ??
		embeddingModels?.[0];

	const startEmbeddingModelDownload = async () => {
		if (!selectedEmbeddingModel) {
			setErrorMessage('No local search model is available yet.');
			return;
		}

		setIsStartingModelDownload(true);
		setStatusMessage(null);
		setErrorMessage(null);

		try {
			await setValues({
				[SEARCH_EMBEDDINGS_ENABLED_KEY]: 'true',
				[SEARCH_EMBEDDINGS_PROVIDER_KEY]: 'local',
				[SEARCH_EMBEDDINGS_MODEL_KEY]: selectedEmbeddingModel.name,
				[SEARCH_EMBEDDINGS_AUTO_INDEX_KEY]: 'true',
			});
			await downloadSearchEmbeddingModel(selectedEmbeddingModel.name);
			setStatusMessage('Local search model download started.');
		} catch (error) {
			setErrorMessage(getErrorMessage(error));
		} finally {
			setIsStartingModelDownload(false);
		}
	};

	const rebuildEmbeddings = async () => {
		if (!selectedEmbeddingModel) return;

		setIsIndexingEmbeddings(true);
		setStatusMessage(null);
		setErrorMessage(null);

		try {
			await reindexSearchDocuments();
			const status = await indexSearchEmbeddings(selectedEmbeddingModel.name);
			setStatusMessage(`${status.totalEmbeddings} local search embeddings indexed.`);
		} catch (error) {
			setErrorMessage(getErrorMessage(error));
		} finally {
			setIsIndexingEmbeddings(false);
		}
	};

	return (
		<div className='w-full space-y-8'>
			<div>
				<h3 className='flex items-center gap-2 text-lg font-medium'>Search</h3>
				<p className='mt-2 text-sm text-(--color-secondary-text)'>
					Set up the local model used for journal and task search.
				</p>
			</div>

			<div className='rounded-lg bg-(--color-panel) p-4 shadow-xs ring ring-neutral-200/40'>
				<div className='mb-4 flex items-center justify-between gap-4'>
					<div>
						<p className='flex items-center gap-2 text-sm font-medium'>
							<HardDrive className='size-4' />
							Local search model
						</p>
						<p className='mt-1 max-w-xl text-xs leading-5 text-(--color-secondary-text)'>
							Stored on this device and used to index journal entries and tasks.
						</p>
					</div>
					<div
						className={cn(
							'inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs',
							selectedEmbeddingModel?.isDownloaded
								? 'border-emerald-500/30 bg-emerald-500/30 text-emerald-700'
								: 'border-(--color-border) text-(--color-secondary-text)',
						)}
					>
						{selectedEmbeddingModel?.isDownloaded ? (
							<CheckCircle2 className='size-3.5' />
						) : (
							<CircleAlert className='size-3.5' />
						)}
						{selectedEmbeddingModel?.isDownloaded ? 'Downloaded' : 'Not downloaded'}
					</div>
				</div>

				<div className='grid gap-3 text-sm md:grid-cols-2'>
					<div className='flex flex-col justify-between rounded-lg border border-(--color-border)/30 bg-neutral-200/50 p-3'>
						<p className='font-medium'>{selectedEmbeddingModel?.name ?? 'No model available'}</p>
						<p className='mt-1 text-xs text-(--color-secondary-text)'>
							{selectedEmbeddingModel
								? `${formatModelSize(selectedEmbeddingModel.fileSize)}${
										selectedEmbeddingModel.dimensions
											? `, ${selectedEmbeddingModel.dimensions} dimensions`
											: ''
									}`
								: 'Model catalog unavailable'}
						</p>
					</div>
					<div className='flex flex-col justify-between rounded-lg border border-(--color-border)/30 bg-neutral-200/50 p-3'>
						<p className='font-medium'>Storage</p>
						<p className='mt-1 text-xs break-all text-(--color-secondary-text)'>
							{selectedEmbeddingModel?.modelPath ??
								selectedEmbeddingModel?.modelsDirectory ??
								'Not downloaded yet.'}
						</p>
					</div>
				</div>

				<div className='mt-4 flex flex-wrap justify-end gap-2'>
					<Button
						onClick={() => {
							void refetchEmbeddingModels();
						}}
						label='Refresh'
						tooltipContent='Refresh local model status'
						variant='secondary'
						iconLeft={<RefreshCw className='size-4' />}
					/>
					<Button
						onClick={rebuildEmbeddings}
						label={isIndexingEmbeddings ? 'Indexing...' : 'Rebuild index'}
						tooltipContent='Rebuild local search embeddings'
						variant='secondary'
						isDisabled={
							isIndexingEmbeddings ||
							isStartingModelDownload ||
							!selectedEmbeddingModel?.isDownloaded
						}
					/>
					<Button
						onClick={startEmbeddingModelDownload}
						label={
							selectedEmbeddingModel?.isDownloaded
								? 'Downloaded'
								: isStartingModelDownload
									? 'Starting...'
									: 'Download now'
						}
						tooltipContent='Download the local search model'
						isDisabled={isStartingModelDownload || Boolean(selectedEmbeddingModel?.isDownloaded)}
						iconLeft={<Download className='size-4' />}
					/>
				</div>
			</div>

			{statusMessage && (
				<div className='rounded-lg border border-emerald-500/30 bg-emerald-500/10 p-3 text-sm text-emerald-700'>
					{statusMessage}
				</div>
			)}
			{errorMessage && (
				<div className='rounded-lg border border-red-500/30 bg-red-500/10 p-3 text-sm text-red-600'>
					{errorMessage}
				</div>
			)}
		</div>
	);
};
