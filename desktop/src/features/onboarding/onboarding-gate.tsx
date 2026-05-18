import { useQuery, useQueryClient } from '@tanstack/react-query';
import { AnimatePresence, motion } from 'motion/react';
import { type FormEvent, type ReactNode, useMemo, useState } from 'react';
import {
	configureSync,
	getGetSyncStatusQueryKey,
	getListProvidersQueryKey,
	useListProviders,
	validateProvider,
} from '~/aether-sdk';
import {
	DEFAULT_SEARCH_EMBEDDING_MODEL,
	downloadSearchEmbeddingModel,
	listSearchEmbeddingModels,
} from '~/lib/search-embedding-models';
import { AiStep } from './components/ai-step';
import { IntroStep } from './components/intro-step';
import { OnboardingFooter } from './components/onboarding-footer';
import { OnboardingPreview } from './components/onboarding-preview';
import { OnboardingStepHeader } from './components/onboarding-step-header';
import { ProfileStep } from './components/profile-step';
import { SyncStep } from './components/sync-step';
import { useSettingsStore } from '~/store/settings-store';
import {
	DEFAULT_PROVIDER_KEY,
	DISPLAY_NAME_KEY,
	GROQ_API_KEY,
	ONBOARDING_COMPLETED_KEY,
	OPENAI_API_KEY,
	RECOVERY_SEED_KEY,
	SEARCH_EMBEDDINGS_AUTO_INDEX_KEY,
	SEARCH_EMBEDDINGS_ENABLED_KEY,
	SEARCH_EMBEDDINGS_MODEL_KEY,
	SEARCH_EMBEDDINGS_PROVIDER_KEY,
	providerCopy,
	recoveryWords,
	steps,
} from './onboarding.constants';
import { type AiChoice, type ProviderChoice, type SyncChoice } from './onboarding.types';

interface OnboardingGateProps {
	children: ReactNode;
}

function isConfigured(value: string) {
	return value.trim().length > 0;
}

function getErrorMessage(error: unknown) {
	if (error && typeof error === 'object' && 'data' in error) {
		const data = (error as { data?: unknown }).data;
		if (data && typeof data === 'object' && 'message' in data) {
			return String((data as { message?: unknown }).message);
		}
	}
	return error instanceof Error ? error.message : String(error);
}

function generateRecoverySeed() {
	const values = new Uint32Array(12);
	crypto.getRandomValues(values);
	return Array.from(values, value => recoveryWords[value % recoveryWords.length]).join(' ');
}

export function OnboardingGate({ children }: OnboardingGateProps) {
	const queryClient = useQueryClient();
	const [stepIndex, setStepIndex] = useState(0);
	const [displayName, setDisplayName] = useState('');
	const [recoverySeed, setRecoverySeed] = useState('');
	const [syncChoice, setSyncChoice] = useState<SyncChoice>(null);
	const [serverUrl, setServerUrl] = useState('');
	const [serverSeedPhrase, setServerSeedPhrase] = useState('');
	const [syncPassphrase, setSyncPassphrase] = useState('');
	const [aiChoice, setAiChoice] = useState<AiChoice>(null);
	const [provider, setProvider] = useState<ProviderChoice>('openai');
	const [openaiKey, setOpenaiKey] = useState('');
	const [groqKey, setGroqKey] = useState('');
	const [isSaving, setIsSaving] = useState(false);
	const [isStartingModelDownload, setIsStartingModelDownload] = useState(false);
	const [statusMessage, setStatusMessage] = useState<string | null>(null);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);
	const { settings, setValue, isLoading } = useSettingsStore();

	const { data: providersResponse } = useListProviders({
		query: {
			queryKey: getListProvidersQueryKey(),
		},
	});

	const { data: embeddingModels } = useQuery({
		queryKey: ['search-embedding-models'],
		queryFn: listSearchEmbeddingModels,
	});
	const isComplete = settings[ONBOARDING_COMPLETED_KEY] === 'true';
	const currentStep = steps[stepIndex];
	const defaultEmbeddingModel =
		embeddingModels?.find(model => model.name === DEFAULT_SEARCH_EMBEDDING_MODEL) ??
		embeddingModels?.[0];
	const activeApiKey = provider === 'openai' ? openaiKey : groqKey;
	const hasAnyApiKey = isConfigured(openaiKey) || isConfigured(groqKey);
	const providerStatuses = new Map(
		(providersResponse?.data ?? []).map(item => [item.name, item.status]),
	);
	const progress = ((stepIndex + 1) / steps.length) * 100;
	const canContinueCurrentStep =
		currentStep.id !== 'sync' ||
		syncChoice === 'no' ||
		(syncChoice === 'yes' &&
			serverUrl.trim().length > 0 &&
			serverSeedPhrase.trim().length >= 12 &&
			syncPassphrase.trim().length >= 12);

	const previewItems = useMemo(
		() => [
			{
				label: 'Name',
				value: displayName.trim() || 'Local-first journal',
				active: stepIndex >= 1,
			},
			{
				label: 'Recovery',
				value: recoverySeed ? 'Seed phrase generated' : 'Optional',
				active: Boolean(recoverySeed),
			},
			{
				label: 'Sync',
				value:
					syncChoice === 'yes'
						? serverUrl.trim() || 'Server details needed'
						: syncChoice === 'no'
							? 'Setup guide ready'
							: 'Decide later',
				active: syncChoice !== null,
			},
			{
				label: 'AI',
				value: defaultEmbeddingModel?.isDownloaded
					? 'Local search ready'
					: isStartingModelDownload
						? 'Local model download started'
						: aiChoice === 'yes'
							? providerCopy[provider].label
							: aiChoice === 'no'
								? 'Off for now'
								: 'Optional',
				active:
					aiChoice !== null ||
					Boolean(defaultEmbeddingModel?.isDownloaded) ||
					isStartingModelDownload,
			},
		],
		[
			aiChoice,
			defaultEmbeddingModel?.isDownloaded,
			displayName,
			isStartingModelDownload,
			provider,
			recoverySeed,
			serverUrl,
			stepIndex,
			syncChoice,
		],
	);

	if (isLoading) {
		return (
			<div className='grid h-screen w-screen place-items-center bg-(--color-background)'>
				<div className='text-sm text-(--color-secondary-text)'>Loading Aether...</div>
			</div>
		);
	}

	if (isComplete) return <>{children}</>;

	const goToStep = (index: number) => {
		setErrorMessage(null);
		setStatusMessage(null);
		setStepIndex(Math.max(0, Math.min(index, steps.length - 1)));
	};

	const validateSyncStep = () => {
		if (syncChoice === null) throw new Error('Choose whether to connect sync now or later.');
		if (syncChoice !== 'yes') return;
		if (!serverUrl.trim()) throw new Error('Add your sync server URL first.');
		if (serverSeedPhrase.trim().length < 12) {
			throw new Error('Server seed phrase must be at least 12 characters.');
		}
		if (syncPassphrase.trim().length < 12) {
			throw new Error('Sync passphrase must be at least 12 characters.');
		}
	};

	const nextStep = () => {
		setErrorMessage(null);
		setStatusMessage(null);
		try {
			if (currentStep.id === 'sync') validateSyncStep();
			goToStep(stepIndex + 1);
		} catch (error) {
			setErrorMessage(getErrorMessage(error));
		}
	};

	const saveCoreSettings = async () => {
		const trimmedName = displayName.trim();
		if (trimmedName) {
			await setValue(DISPLAY_NAME_KEY, trimmedName);
		}
		if (recoverySeed) {
			await setValue(RECOVERY_SEED_KEY, recoverySeed);
		}
		if (syncChoice === 'yes') {
			validateSyncStep();
			await configureSync({
				server_url: serverUrl.trim(),
				server_seed_phrase: serverSeedPhrase.trim(),
				sync_passphrase: syncPassphrase.trim(),
			});
		}
	};

	const completeOnboarding = async (shouldValidateProvider: boolean) => {
		setIsSaving(true);
		setErrorMessage(null);
		setStatusMessage(null);

		try {
			await saveCoreSettings();
			if (aiChoice === 'yes') {
				if (!hasAnyApiKey) {
					throw new Error(`Add a ${providerCopy[provider].keyLabel} first, or choose no AI.`);
				}
				if (isConfigured(openaiKey)) {
					await setValue(OPENAI_API_KEY, openaiKey.trim());
				}
				if (isConfigured(groqKey)) {
					await setValue(GROQ_API_KEY, groqKey.trim());
				}
				await setValue(DEFAULT_PROVIDER_KEY, provider);
				if (shouldValidateProvider) {
					await validateProvider({
						body: JSON.stringify({ provider_name: provider }),
						headers: { 'Content-Type': 'application/json' },
					});
				}
			}
			await setValue(ONBOARDING_COMPLETED_KEY, 'true');
			await queryClient.invalidateQueries({ queryKey: getListProvidersQueryKey() });
			await queryClient.invalidateQueries({ queryKey: getGetSyncStatusQueryKey() });
		} catch (error) {
			setErrorMessage(getErrorMessage(error));
			setStatusMessage(null);
		} finally {
			setIsSaving(false);
		}
	};

	const validateSelectedProvider = async () => {
		setIsSaving(true);
		setErrorMessage(null);
		setStatusMessage(null);

		try {
			if (!isConfigured(activeApiKey)) {
				throw new Error(`Add a ${providerCopy[provider].keyLabel} first.`);
			}
			await setValue(provider === 'openai' ? OPENAI_API_KEY : GROQ_API_KEY, activeApiKey.trim());
			await setValue(DEFAULT_PROVIDER_KEY, provider);
			await validateProvider({
				body: JSON.stringify({ provider_name: provider }),
				headers: { 'Content-Type': 'application/json' },
			});
			await queryClient.invalidateQueries({ queryKey: getListProvidersQueryKey() });
			setStatusMessage(`${providerCopy[provider].label} is ready.`);
		} catch (error) {
			setErrorMessage(getErrorMessage(error));
		} finally {
			setIsSaving(false);
		}
	};

	const startEmbeddingModelDownload = async () => {
		if (!defaultEmbeddingModel) {
			setErrorMessage('No local search model is available yet.');
			return;
		}

		setIsStartingModelDownload(true);
		setErrorMessage(null);
		setStatusMessage(null);

		try {
			await setValue(SEARCH_EMBEDDINGS_ENABLED_KEY, 'true');
			await setValue(SEARCH_EMBEDDINGS_PROVIDER_KEY, 'local');
			await setValue(SEARCH_EMBEDDINGS_MODEL_KEY, defaultEmbeddingModel.name);
			await setValue(SEARCH_EMBEDDINGS_AUTO_INDEX_KEY, 'true');
			await downloadSearchEmbeddingModel(defaultEmbeddingModel.name);
			setStatusMessage('Local search model download started. You can finish onboarding now.');
		} catch (error) {
			setErrorMessage(getErrorMessage(error));
		} finally {
			setIsStartingModelDownload(false);
		}
	};

	const onSubmit = (event: FormEvent<HTMLFormElement>) => {
		event.preventDefault();
		if (stepIndex === steps.length - 1) {
			completeOnboarding(false);
			return;
		}
		nextStep();
	};

	return (
		<div className='min-h-screen w-screen overflow-y-auto bg-[linear-gradient(135deg,var(--color-background),var(--color-background-secondary))] px-5 py-8 text-(--color-foreground)'>
			<div className='mx-auto grid min-h-[calc(100vh-4rem)] max-w-6xl grid-cols-1 items-center gap-6 lg:grid-cols-[0.92fr_1.08fr]'>
				<OnboardingPreview items={previewItems} />

				<form
					onSubmit={onSubmit}
					className='min-h-[520px] rounded-lg border border-(--color-border) bg-(--color-card) p-5 shadow-2xl shadow-black/10 md:p-7'
				>
					<div className='flex min-h-[464px] flex-col'>
						<OnboardingStepHeader
							stepIndex={stepIndex}
							progress={progress}
							onStepChange={goToStep}
						/>

						<div className='flex-1 py-6'>
							<AnimatePresence mode='wait'>
								<motion.div
									key={currentStep.id}
									initial={{ opacity: 0, y: 12 }}
									animate={{ opacity: 1, y: 0 }}
									exit={{ opacity: 0, y: -8 }}
									transition={{ duration: 0.22 }}
								>
									{currentStep.id === 'intro' && <IntroStep />}
									{currentStep.id === 'profile' && (
										<ProfileStep
											displayName={displayName}
											recoverySeed={recoverySeed}
											onDisplayNameChange={setDisplayName}
											onGenerateRecoverySeed={() => setRecoverySeed(generateRecoverySeed())}
										/>
									)}
									{currentStep.id === 'sync' && (
										<SyncStep
											syncChoice={syncChoice}
											serverUrl={serverUrl}
											serverSeedPhrase={serverSeedPhrase}
											syncPassphrase={syncPassphrase}
											onSyncChoiceChange={setSyncChoice}
											onServerUrlChange={setServerUrl}
											onServerSeedPhraseChange={setServerSeedPhrase}
											onSyncPassphraseChange={setSyncPassphrase}
										/>
									)}
									{currentStep.id === 'ai' && (
										<AiStep
											aiChoice={aiChoice}
											provider={provider}
											activeApiKey={activeApiKey}
											providerStatuses={providerStatuses}
											defaultEmbeddingModel={defaultEmbeddingModel}
											isStartingModelDownload={isStartingModelDownload}
											onAiChoiceChange={setAiChoice}
											onProviderChange={setProvider}
											onApiKeyChange={value =>
												provider === 'openai' ? setOpenaiKey(value) : setGroqKey(value)
											}
											onStartEmbeddingModelDownload={startEmbeddingModelDownload}
										/>
									)}
								</motion.div>
							</AnimatePresence>
						</div>

						<OnboardingFooter
							stepIndex={stepIndex}
							isSaving={isSaving}
							isAiStep={currentStep.id === 'ai'}
							aiChoice={aiChoice}
							activeApiKey={activeApiKey}
							canContinueCurrentStep={canContinueCurrentStep}
							statusMessage={statusMessage}
							errorMessage={errorMessage}
							onBack={() => goToStep(stepIndex - 1)}
							onContinue={nextStep}
							onComplete={() => completeOnboarding(aiChoice === 'yes')}
							onValidateProvider={validateSelectedProvider}
							isConfigured={isConfigured}
						/>
					</div>
				</form>
			</div>
		</div>
	);
}
