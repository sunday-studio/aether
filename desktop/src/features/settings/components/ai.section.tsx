import { useQuery, useQueryClient } from '@tanstack/react-query';
import { CheckCircle2, CircleAlert, Download, HardDrive, KeyRound, RefreshCw } from 'lucide-react';
import { useState } from 'react';
import { getListProvidersQueryKey, useListProviders, validateProvider } from '~/aether-sdk';
import { Button } from '~/components/shared/button';
import { Select, SelectItem } from '~/components/shared/select';
import { TextField } from '~/components/shared/text-field';
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

type ProviderChoice = 'openai' | 'groq';

const DEFAULT_PROVIDER_KEY = 'transcription.default_provider';
const OPENAI_API_KEY = 'transcription.openai.api_key';
const GROQ_API_KEY = 'transcription.groq.api_key';
const SEARCH_EMBEDDINGS_ENABLED_KEY = 'search.embeddings.enabled';
const SEARCH_EMBEDDINGS_PROVIDER_KEY = 'search.embeddings.provider';
const SEARCH_EMBEDDINGS_MODEL_KEY = 'search.embeddings.model';
const SEARCH_EMBEDDINGS_AUTO_INDEX_KEY = 'search.embeddings.auto_index';

const providerCopy: Record<
	ProviderChoice,
	{ label: string; keyLabel: string; placeholder: string }
> = {
	openai: {
		label: 'OpenAI Whisper',
		keyLabel: 'OpenAI API Key',
		placeholder: 'sk-...',
	},
	groq: {
		label: 'Groq',
		keyLabel: 'Groq API Key',
		placeholder: 'gsk_...',
	},
};

function getErrorMessage(error: unknown) {
	if (error && typeof error === 'object' && 'data' in error) {
		const data = (error as { data?: unknown }).data;
		if (data && typeof data === 'object' && 'message' in data) {
			return String((data as { message?: unknown }).message);
		}
	}
	return error instanceof Error ? error.message : String(error);
}

function isConfigured(value: unknown) {
	return typeof value === 'string' && value.trim().length > 0;
}

function providerStatusLabel(status?: string) {
	if (!status) return 'Unknown';
	if (status === 'ready') return 'Ready';
	if (status === 'not_configured') return 'Not configured';
	if (status.startsWith('error:')) return 'Error';
	return status;
}

export const AiSection = () => {
	const queryClient = useQueryClient();
	const [provider, setProvider] = useState<ProviderChoice | null>(null);
	const [openaiKey, setOpenaiKey] = useState('');
	const [groqKey, setGroqKey] = useState('');
	const [isSaving, setIsSaving] = useState(false);
	const [isStartingModelDownload, setIsStartingModelDownload] = useState(false);
	const [isIndexingEmbeddings, setIsIndexingEmbeddings] = useState(false);
	const [statusMessage, setStatusMessage] = useState<string | null>(null);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);
	const { settings, setValue, setValues } = useSettingsStore();
	const { data: providersResponse } = useListProviders({
		query: { queryKey: getListProvidersQueryKey() },
	});
	const { data: embeddingModels, refetch: refetchEmbeddingModels } = useQuery({
		queryKey: ['search-embedding-models'],
		queryFn: listSearchEmbeddingModels,
	});
	const savedProvider = (settings[DEFAULT_PROVIDER_KEY] as ProviderChoice | undefined) ?? 'openai';
	const selectedEmbeddingModel =
		embeddingModels?.find(model => model.name === DEFAULT_SEARCH_EMBEDDING_MODEL) ??
		embeddingModels?.[0];
	const selectedProvider = provider ?? savedProvider;
	const providers = providersResponse?.data ?? [];
	const providerStatuses = new Map(providers.map(item => [item.name, item]));
	const activeProvider = providerStatuses.get(selectedProvider);
	const hasOpenAiKey = isConfigured(settings[OPENAI_API_KEY]);
	const hasGroqKey = isConfigured(settings[GROQ_API_KEY]);
	const activeApiKey = selectedProvider === 'openai' ? openaiKey : groqKey;
	const selectedProviderHasSavedKey = selectedProvider === 'openai' ? hasOpenAiKey : hasGroqKey;

	const invalidateAiQueries = async () => {
		await queryClient.invalidateQueries({ queryKey: getListProvidersQueryKey() });
	};

	const saveKeysAndProvider = async () => {
		setIsSaving(true);
		setStatusMessage(null);
		setErrorMessage(null);

		try {
			await setValues({
				[DEFAULT_PROVIDER_KEY]: selectedProvider,
				...(openaiKey.trim() ? { [OPENAI_API_KEY]: openaiKey.trim() } : {}),
				...(groqKey.trim() ? { [GROQ_API_KEY]: groqKey.trim() } : {}),
			});
			await invalidateAiQueries();
			setOpenaiKey('');
			setGroqKey('');
			setStatusMessage('AI transcription settings saved.');
		} catch (error) {
			setErrorMessage(getErrorMessage(error));
		} finally {
			setIsSaving(false);
		}
	};

	const validateSelectedProvider = async () => {
		setIsSaving(true);
		setStatusMessage(null);
		setErrorMessage(null);

		try {
			if (activeApiKey.trim()) {
				await setValue(
					selectedProvider === 'openai' ? OPENAI_API_KEY : GROQ_API_KEY,
					activeApiKey.trim(),
				);
			}
			await setValue(DEFAULT_PROVIDER_KEY, selectedProvider);
			await validateProvider({
				body: JSON.stringify({ provider_name: selectedProvider }),
				headers: { 'Content-Type': 'application/json' },
			});
			await invalidateAiQueries();
			setStatusMessage(`${providerCopy[selectedProvider].label} is ready.`);
		} catch (error) {
			setErrorMessage(getErrorMessage(error));
		} finally {
			setIsSaving(false);
		}
	};

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
			await invalidateAiQueries();
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
				<h3 className='flex items-center gap-2 text-lg font-medium'>AI</h3>
				<p className='mt-2 text-sm text-(--color-secondary-text)'>
					AI is optional for v1. Manage the local search model and hosted transcription keys here.
				</p>
			</div>

			<div className='rounded-lg bg-(--color-panel) p-4 shadow-xs ring ring-neutral-200/40'>
				<div className='mb-4 flex flex-wrap items-start justify-between gap-4'>
					<div>
						<p className='flex items-center gap-2 text-sm font-medium'>
							<HardDrive className='size-4' />
							Local search model
						</p>
						<p className='mt-1 max-w-xl text-xs leading-5 text-(--color-secondary-text)'>
							Download a local embedding model for semantic search and related context. It runs on
							this device and is not synced.
						</p>
					</div>
					<div
						className={cn(
							'inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs',
							selectedEmbeddingModel?.isDownloaded
								? 'border-emerald-500/30 bg-emerald-500/10 text-emerald-700'
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
					<div className='flex flex-col justify-between rounded-lg border border-(--color-border) bg-neutral-200/50 p-3'>
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
					<div className='flex flex-col justify-between rounded-lg border border-(--color-border) bg-neutral-200/50 p-3'>
						<p className='font-medium'>Storage</p>
						<p className='mt-1 text-xs break-all text-(--color-secondary-text)'>
							{selectedEmbeddingModel?.modelPath ??
								selectedEmbeddingModel?.modelsDirectory ??
								'Download the model to create the local model directory.'}
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
						label={isIndexingEmbeddings ? 'Indexing...' : 'Rebuild embeddings'}
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

			<div className='rounded-lg bg-(--color-panel) p-4 shadow-xs ring ring-neutral-200/40'>
				<div className='mb-4 flex items-start justify-between gap-4'>
					<div>
						<p className='text-sm font-medium'>Provider status</p>
						<p className='text-xs text-(--color-secondary-text)'>
							Default provider: {providerCopy[savedProvider]?.label ?? savedProvider}
						</p>
					</div>
					<div
						className={cn(
							'inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs',
							activeProvider?.status === 'not_configured' ? 'bg-neutral-200/50' : '',
							activeProvider?.status === 'ready'
								? 'border-emerald-500/30 bg-emerald-500/10 text-emerald-700'
								: '',
							activeProvider?.status !== 'ready' && activeProvider?.status !== 'not_configured'
								? 'border-(--color-border) text-(--color-secondary-text)'
								: '',
						)}
					>
						{activeProvider?.status === 'ready' ? (
							<CheckCircle2 className='size-3.5' />
						) : (
							<CircleAlert className='size-3.5' />
						)}
						{providerStatusLabel(activeProvider?.status)}
					</div>
				</div>

				<div className='grid gap-3 text-sm sm:grid-cols-2'>
					<div className='rounded-lg border border-(--color-border) bg-(--color-background) p-3'>
						<p className='font-medium'>OpenAI</p>
						<p className='mt-1 text-xs text-(--color-secondary-text)'>
							{hasOpenAiKey ? 'Key saved' : 'No key saved'}
						</p>
					</div>
					<div className='rounded-lg border border-(--color-border) bg-(--color-background) p-3'>
						<p className='font-medium'>Groq</p>
						<p className='mt-1 text-xs text-(--color-secondary-text)'>
							{hasGroqKey ? 'Key saved' : 'No key saved'}
						</p>
					</div>
				</div>
			</div>

			<div className='space-y-4'>
				<Select
					label='Default transcription provider'
					placeholder='Choose provider'
					value={selectedProvider}
					onChange={value => setProvider(value as ProviderChoice)}
					items={[
						{ label: providerCopy.openai.label, value: 'openai' },
						{ label: providerCopy.groq.label, value: 'groq' },
					]}
				>
					<SelectItem id='openai'>OpenAI Whisper</SelectItem>
					<SelectItem id='groq'>Groq</SelectItem>
				</Select>

				<div className='grid gap-4 sm:grid-cols-2'>
					<TextField
						label={providerCopy.openai.keyLabel}
						placeholder={hasOpenAiKey ? 'Key saved' : providerCopy.openai.placeholder}
						type='password'
						value={openaiKey}
						onChange={value => setOpenaiKey(value)}
					/>
					<TextField
						label={providerCopy.groq.keyLabel}
						placeholder={hasGroqKey ? 'Key saved' : providerCopy.groq.placeholder}
						type='password'
						value={groqKey}
						onChange={value => setGroqKey(value)}
					/>
				</div>

				<p className='flex items-start gap-2 text-xs leading-5 text-(--color-secondary-text)'>
					<KeyRound className='mt-0.5 size-3.5 shrink-0' />
					Keys are stored through the encrypted settings path. Leave a field blank to keep the
					existing saved key unchanged.
				</p>
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

			<div className='flex justify-end gap-3'>
				<Button
					onClick={saveKeysAndProvider}
					label={isSaving ? 'Saving...' : 'Save'}
					tooltipContent='Save AI transcription settings'
					isDisabled={isSaving}
				/>
				<Button
					onClick={validateSelectedProvider}
					label={isSaving ? 'Checking...' : 'Validate'}
					tooltipContent='Validate selected provider'
					isDisabled={isSaving || (!activeApiKey.trim() && !selectedProviderHasSavedKey)}
				/>
			</div>
		</div>
	);
};
