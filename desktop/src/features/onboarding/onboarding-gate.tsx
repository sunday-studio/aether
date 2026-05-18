import { useQueryClient } from '@tanstack/react-query';
import {
	ArrowLeft,
	ArrowRight,
	Check,
	CheckCircle2,
	Cloud,
	Copy,
	ExternalLink,
	KeyRound,
	Server,
	Sparkles,
	UserRound,
	WandSparkles,
} from 'lucide-react';
import { AnimatePresence, motion } from 'motion/react';
import { type FormEvent, type ReactNode, useMemo, useState } from 'react';
import {
	configureSync,
	getGetAllSettingsQueryKey,
	getGetSyncStatusQueryKey,
	getListProvidersQueryKey,
	setSetting,
	useGetAllSettings,
	useListProviders,
	validateProvider,
} from '~/aether-sdk';
import { Button } from '~/components/shared/button';
import { TextField } from '~/components/shared/text-field';
import { cn } from '~/utils/cn';

const ONBOARDING_COMPLETED_KEY = 'app.onboarding_completed';
const DISPLAY_NAME_KEY = 'user.display_name';
const RECOVERY_SEED_KEY = 'user.recovery_seed_phrase';
const DEFAULT_PROVIDER_KEY = 'transcription.default_provider';
const OPENAI_API_KEY = 'transcription.openai.api_key';
const GROQ_API_KEY = 'transcription.groq.api_key';
const SYNC_GUIDE_URL =
	'https://github.com/sunday-studio/aether/blob/main/docs/reference/sync-server-readme.md';

type ProviderChoice = 'openai' | 'groq';
type StepId = 'intro' | 'profile' | 'sync' | 'ai';
type SyncChoice = 'yes' | 'no' | null;
type AiChoice = 'yes' | 'no' | null;

interface OnboardingGateProps {
	children: ReactNode;
}

const recoveryWords = [
	'aether',
	'anchor',
	'archive',
	'atelier',
	'bloom',
	'canvas',
	'cipher',
	'compass',
	'ember',
	'field',
	'glimmer',
	'harbor',
	'journal',
	'kernel',
	'lantern',
	'ledger',
	'meadow',
	'notebook',
	'orbit',
	'parcel',
	'quartz',
	'river',
	'signal',
	'studio',
	'thread',
	'vault',
	'velvet',
	'window',
];

const providerCopy: Record<
	ProviderChoice,
	{ label: string; description: string; keyLabel: string; placeholder: string }
> = {
	openai: {
		label: 'OpenAI Whisper',
		description: 'Reliable hosted transcription if you already use OpenAI.',
		keyLabel: 'OpenAI API Key',
		placeholder: 'sk-...',
	},
	groq: {
		label: 'Groq',
		description: "Fast hosted transcription with Groq's API.",
		keyLabel: 'Groq API Key',
		placeholder: 'gsk_...',
	},
};

const steps: Array<{ id: StepId; label: string; icon: typeof Sparkles }> = [
	{ id: 'intro', label: 'Welcome', icon: Sparkles },
	{ id: 'profile', label: 'Identity', icon: UserRound },
	{ id: 'sync', label: 'Sync', icon: Server },
	{ id: 'ai', label: 'AI', icon: WandSparkles },
];

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

function saveSetting(key: string, value: string) {
	return setSetting({ key, value });
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
	const [statusMessage, setStatusMessage] = useState<string | null>(null);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const { data: settingsResponse, isLoading } = useGetAllSettings({
		query: {
			queryKey: getGetAllSettingsQueryKey(),
		},
	});

	const { data: providersResponse } = useListProviders({
		query: {
			queryKey: getListProvidersQueryKey(),
		},
	});

	const settings = settingsResponse?.data ?? {};
	const isComplete = settings[ONBOARDING_COMPLETED_KEY] === 'true';
	const currentStep = steps[stepIndex];
	const activeApiKey = provider === 'openai' ? openaiKey : groqKey;
	const hasAnyApiKey = isConfigured(openaiKey) || isConfigured(groqKey);
	const providerStatuses = new Map(
		(providersResponse?.data ?? []).map(item => [item.name, item.status]),
	);
	const progress = ((stepIndex + 1) / steps.length) * 100;
	const isLastStep = stepIndex === steps.length - 1;
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
				value:
					aiChoice === 'yes'
						? providerCopy[provider].label
						: aiChoice === 'no'
							? 'Off for now'
							: 'Optional',
				active: aiChoice !== null,
			},
		],
		[aiChoice, displayName, provider, recoverySeed, serverUrl, stepIndex, syncChoice],
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
			await saveSetting(DISPLAY_NAME_KEY, trimmedName);
		}
		if (recoverySeed) {
			await saveSetting(RECOVERY_SEED_KEY, recoverySeed);
		}
		if (syncChoice === 'yes') {
			validateSyncStep();
			await configureSync({
				data: {
					server_url: serverUrl.trim(),
					server_seed_phrase: serverSeedPhrase.trim(),
					sync_passphrase: syncPassphrase.trim(),
				},
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
					await saveSetting(OPENAI_API_KEY, openaiKey.trim());
				}
				if (isConfigured(groqKey)) {
					await saveSetting(GROQ_API_KEY, groqKey.trim());
				}
				await saveSetting(DEFAULT_PROVIDER_KEY, provider);
				if (shouldValidateProvider) {
					await validateProvider({
						body: JSON.stringify({ provider_name: provider }),
						headers: { 'Content-Type': 'application/json' },
					});
				}
			}
			await saveSetting(ONBOARDING_COMPLETED_KEY, 'true');
			await queryClient.invalidateQueries({
				queryKey: getGetAllSettingsQueryKey(),
			});
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
			await saveSetting(provider === 'openai' ? OPENAI_API_KEY : GROQ_API_KEY, activeApiKey.trim());
			await saveSetting(DEFAULT_PROVIDER_KEY, provider);
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
				<section className='relative min-h-[520px] overflow-hidden rounded-lg border border-(--color-border) bg-(--color-card) p-8 shadow-2xl shadow-black/10'>
					<div className='absolute inset-0 bg-[radial-gradient(circle_at_20%_18%,rgba(48,164,108,0.16),transparent_30%),radial-gradient(circle_at_80%_68%,rgba(59,130,246,0.12),transparent_28%)]' />
					<motion.div
						aria-hidden
						className='absolute top-16 right-12 size-44 rounded-full border border-(--color-border)'
						animate={{ rotate: 360 }}
						transition={{ duration: 28, repeat: Infinity, ease: 'linear' }}
					>
						<div className='absolute top-0 left-1/2 size-3 -translate-x-1/2 rounded-full bg-(--color-active-text)' />
						<div className='absolute bottom-6 left-5 size-2 rounded-full bg-sky-500' />
					</motion.div>
					<div className='relative z-10 flex h-full min-h-[456px] flex-col justify-between'>
						<div>
							<div className='inline-flex items-center gap-2 rounded-full border border-(--color-border) bg-(--color-background)/80 px-3 py-1 text-xs text-(--color-secondary-text)'>
								<Sparkles className='size-3.5' />
								First launch setup
							</div>
							<h1 className='newsreader-font mt-8 max-w-lg text-5xl leading-[0.96] md:text-7xl'>
								Shape your quiet place before it starts listening.
							</h1>
							<p className='mt-5 max-w-md text-sm leading-6 text-(--color-secondary-text)'>
								Aether stays local by default. This flow names the space, prepares recovery, and
								connects sync if you already run it. AI remains optional.
							</p>
						</div>

						<div className='grid gap-3'>
							{previewItems.map((item, index) => (
								<motion.div
									key={item.label}
									className='flex items-center justify-between rounded-lg border border-(--color-border) bg-(--color-background)/80 px-4 py-3 text-sm backdrop-blur'
									initial={{ opacity: 0, x: -10 }}
									animate={{ opacity: 1, x: 0 }}
									transition={{ delay: index * 0.06 }}
								>
									<div>
										<p className='font-medium'>{item.label}</p>
										<p className='mt-0.5 text-xs text-(--color-secondary-text)'>{item.value}</p>
									</div>
									<span
										className={cn(
											'grid size-6 place-items-center rounded-full border border-(--color-border)',
											item.active &&
												'border-(--color-active-text) bg-(--color-active-text) text-white',
										)}
									>
										{item.active ? <Check className='size-3.5' /> : index + 1}
									</span>
								</motion.div>
							))}
						</div>
					</div>
				</section>

				<form
					onSubmit={onSubmit}
					className='min-h-[520px] rounded-lg border border-(--color-border) bg-(--color-card) p-5 shadow-2xl shadow-black/10 md:p-7'
				>
					<div className='flex min-h-[464px] flex-col'>
						<div className='space-y-5 border-b border-(--color-border) pb-5'>
							<div className='flex items-center justify-between gap-4'>
								<div>
									<p className='text-xs tracking-[0.2em] text-(--color-secondary-text) uppercase'>
										Step {stepIndex + 1} of {steps.length}
									</p>
									<h2 className='mt-1 text-2xl font-medium'>{currentStep.label}</h2>
								</div>
								<div className='flex items-center gap-1'>
									{steps.map((step, index) => {
										const Icon = step.icon;
										return (
											<button
												key={step.id}
												type='button'
												onClick={() => goToStep(index)}
												className={cn(
													'grid size-9 place-items-center rounded-full border border-(--color-border) text-(--color-secondary-text) transition',
													index === stepIndex &&
														'border-(--color-active-text) bg-(--color-active-text) text-white',
												)}
												aria-label={`Go to ${step.label}`}
											>
												<Icon className='size-4' />
											</button>
										);
									})}
								</div>
							</div>
							<div className='h-1.5 overflow-hidden rounded-full bg-(--color-background-secondary)'>
								<motion.div
									className='h-full rounded-full bg-(--color-active-text)'
									animate={{ width: `${progress}%` }}
									transition={{ type: 'spring', stiffness: 120, damping: 20 }}
								/>
							</div>
						</div>

						<div className='flex-1 py-6'>
							<AnimatePresence mode='wait'>
								<motion.div
									key={currentStep.id}
									initial={{ opacity: 0, y: 12 }}
									animate={{ opacity: 1, y: 0 }}
									exit={{ opacity: 0, y: -8 }}
									transition={{ duration: 0.22 }}
								>
									{currentStep.id === 'intro' && (
										<div className='space-y-6'>
											<div>
												<h3 className='text-3xl font-medium'>Aether is a local-first notebook.</h3>
												<p className='mt-3 max-w-xl text-sm leading-6 text-(--color-secondary-text)'>
													You are about to set up the basics: a name, a recovery seed phrase if you
													want one, optional self-hosted sync, and optional AI keys.
												</p>
											</div>
											<div className='grid gap-3 md:grid-cols-2'>
												<FeaturePill
													icon={UserRound}
													title='Personal'
													copy='Name the workspace without creating an account.'
												/>
												<FeaturePill
													icon={KeyRound}
													title='Recoverable'
													copy='Generate a phrase you can store outside the app.'
												/>
												<FeaturePill
													icon={Cloud}
													title='Self-hosted'
													copy='Bring your own sync server or skip it.'
												/>
												<FeaturePill
													icon={WandSparkles}
													title='AI optional'
													copy='Use hosted AI only when you provide a key.'
												/>
											</div>
										</div>
									)}

									{currentStep.id === 'profile' && (
										<div className='space-y-6'>
											<div>
												<h3 className='text-3xl font-medium'>Start with a name.</h3>
												<p className='mt-3 max-w-xl text-sm leading-6 text-(--color-secondary-text)'>
													The name is local app context. The recovery phrase is optional, but useful
													if you want a memorable seed stored with your setup.
												</p>
											</div>
											<TextField
												label='Display name'
												placeholder='Ada'
												value={displayName}
												onChange={value => setDisplayName(value)}
											/>
											<div className='rounded-lg border border-(--color-border) bg-(--color-background) p-4'>
												<div className='flex flex-wrap items-center justify-between gap-3'>
													<div>
														<p className='text-sm font-medium'>Recovery seed phrase</p>
														<p className='mt-1 text-xs text-(--color-secondary-text)'>
															Generate one now, or leave it empty and continue.
														</p>
													</div>
													<div className='flex gap-2'>
														<button
															type='button'
															onClick={() => setRecoverySeed(generateRecoverySeed())}
															className='rounded-full border border-(--color-border) px-3 py-2 text-xs transition hover:border-(--color-active-text)'
														>
															Generate
														</button>
														<button
															type='button'
															onClick={() =>
																recoverySeed && navigator.clipboard?.writeText(recoverySeed)
															}
															disabled={!recoverySeed}
															className='grid size-8 place-items-center rounded-full border border-(--color-border) text-(--color-secondary-text) transition hover:text-(--color-active-text) disabled:opacity-40'
															aria-label='Copy recovery seed phrase'
														>
															<Copy className='size-4' />
														</button>
													</div>
												</div>
												<p className='mt-4 min-h-12 rounded-lg bg-(--color-card) p-3 text-sm leading-6 text-(--color-secondary-text)'>
													{recoverySeed || 'No seed phrase generated yet.'}
												</p>
											</div>
										</div>
									)}

									{currentStep.id === 'sync' && (
										<div className='space-y-6'>
											<div>
												<h3 className='text-3xl font-medium'>Do you already have a sync server?</h3>
												<p className='mt-3 max-w-xl text-sm leading-6 text-(--color-secondary-text)'>
													Aether sync is end-to-end encrypted and self-hosted. Connect an existing
													server now, or use the setup guide later. The server seed phrase enrolls
													this device; the sync passphrase protects your data before it reaches the
													server.
												</p>
											</div>
											<div className='grid gap-3 md:grid-cols-2'>
												<ChoiceButton
													isSelected={syncChoice === 'yes'}
													onClick={() => setSyncChoice('yes')}
													title='Yes, connect it'
													copy='I have a server URL, server seed phrase, and sync passphrase.'
												/>
												<ChoiceButton
													isSelected={syncChoice === 'no'}
													onClick={() => setSyncChoice('no')}
													title='No, show me how'
													copy='I will set up self-hosted sync later.'
												/>
											</div>
											{syncChoice === 'yes' && (
												<motion.div
													className='grid gap-3'
													initial={{ opacity: 0, height: 0 }}
													animate={{ opacity: 1, height: 'auto' }}
												>
													<TextField
														label='Server URL'
														placeholder='https://your-sync-server:8080'
														value={serverUrl}
														onChange={value => setServerUrl(value)}
													/>
													<div className='grid gap-3 md:grid-cols-2'>
														<TextField
															label='Server seed phrase'
															placeholder='min 12 characters'
															type='password'
															value={serverSeedPhrase}
															onChange={value => setServerSeedPhrase(value)}
														/>
														<TextField
															label='Sync passphrase'
															placeholder='min 12 characters'
															type='password'
															value={syncPassphrase}
															onChange={value => setSyncPassphrase(value)}
														/>
													</div>
													<div className='grid gap-3 text-xs leading-5 text-(--color-secondary-text) md:grid-cols-2'>
														<p>
															The server seed phrase must match your sync server setup. It lets this
															app register as an allowed device.
														</p>
														<p>
															The sync passphrase encrypts local data before upload. The sync server
															cannot recover it.
														</p>
													</div>
												</motion.div>
											)}
											{syncChoice === 'no' && (
												<a
													href={SYNC_GUIDE_URL}
													target='_blank'
													rel='noopener noreferrer'
													className='inline-flex items-center gap-2 rounded-full border border-(--color-border) px-3 py-2 text-sm text-(--color-link) transition hover:border-(--color-active-text)'
												>
													Open sync server setup guide
													<ExternalLink className='size-4' />
												</a>
											)}
										</div>
									)}

									{currentStep.id === 'ai' && (
										<div className='space-y-6'>
											<div>
												<h3 className='text-3xl font-medium'>Use AI in Aether?</h3>
												<p className='mt-3 max-w-xl text-sm leading-6 text-(--color-secondary-text)'>
													AI configuration is optional. Add provider keys now if you want the app
													ready for AI-backed features as they come online.
												</p>
											</div>
											<div className='grid gap-3 md:grid-cols-2'>
												<ChoiceButton
													isSelected={aiChoice === 'yes'}
													onClick={() => setAiChoice('yes')}
													title='Yes, enable AI'
													copy='Choose a provider and save a key.'
												/>
												<ChoiceButton
													isSelected={aiChoice === 'no'}
													onClick={() => setAiChoice('no')}
													title='No, keep it manual'
													copy='Use the app without AI for now.'
												/>
											</div>
											{aiChoice === 'yes' && (
												<motion.div
													className='space-y-4'
													initial={{ opacity: 0, height: 0 }}
													animate={{ opacity: 1, height: 'auto' }}
												>
													<div className='grid gap-3 md:grid-cols-2'>
														{(['openai', 'groq'] as ProviderChoice[]).map(choice => {
															const isSelected = provider === choice;
															const status = providerStatuses.get(choice);

															return (
																<button
																	type='button'
																	key={choice}
																	onClick={() => setProvider(choice)}
																	className={cn(
																		'rounded-lg border border-(--color-border) bg-(--color-background) p-4 text-left transition hover:border-(--color-active-text)',
																		isSelected &&
																			'border-(--color-active-text) bg-(--color-background-secondary)',
																	)}
																>
																	<div className='flex items-center justify-between gap-3'>
																		<p className='text-sm font-medium'>
																			{providerCopy[choice].label}
																		</p>
																		{status === 'ready' && (
																			<span className='rounded-full bg-emerald-500/10 px-2 py-0.5 text-[11px] text-emerald-600'>
																				Ready
																			</span>
																		)}
																	</div>
																	<p className='mt-2 text-xs text-(--color-secondary-text)'>
																		{providerCopy[choice].description}
																	</p>
																</button>
															);
														})}
													</div>
													<TextField
														label={providerCopy[provider].keyLabel}
														placeholder={providerCopy[provider].placeholder}
														type='password'
														value={activeApiKey}
														onChange={value =>
															provider === 'openai' ? setOpenaiKey(value) : setGroqKey(value)
														}
													/>
												</motion.div>
											)}
										</div>
									)}
								</motion.div>
							</AnimatePresence>
						</div>

						<div className='space-y-3 border-t border-(--color-border) pt-5'>
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
							<div className='flex flex-wrap items-center justify-between gap-3'>
								<button
									type='button'
									onClick={() => goToStep(stepIndex - 1)}
									disabled={stepIndex === 0 || isSaving}
									className='inline-flex items-center gap-2 text-sm text-(--color-secondary-text) transition hover:text-(--color-active-text) disabled:opacity-40'
								>
									<ArrowLeft className='size-4' />
									Back
								</button>
								<div className='flex flex-wrap items-center justify-end gap-3'>
									{currentStep.id === 'ai' && aiChoice === 'yes' && (
										<Button
											onClick={validateSelectedProvider}
											label={isSaving ? 'Checking...' : 'Validate provider'}
											tooltipContent='Validate the selected provider credentials'
											isDisabled={isSaving || !isConfigured(activeApiKey)}
										/>
									)}
									{currentStep.id === 'ai' ? (
										<Button
											onClick={() => completeOnboarding(aiChoice === 'yes')}
											label={isSaving ? 'Saving...' : 'Start using Aether'}
											tooltipContent='Finish onboarding'
											isDisabled={isSaving || aiChoice === null}
										/>
									) : (
										<button
											type='submit'
											disabled={isSaving || !canContinueCurrentStep}
											className='inline-flex items-center gap-2 rounded-full bg-(--color-active-text) px-4 py-2.5 text-sm leading-none text-white transition hover:bg-(--color-active-text-hover) disabled:opacity-50'
										>
											{isLastStep ? 'Start using Aether' : 'Continue'}
											<ArrowRight className='size-4' />
										</button>
									)}
								</div>
							</div>
						</div>
					</div>
				</form>
			</div>
		</div>
	);
}

function FeaturePill({
	icon: Icon,
	title,
	copy,
}: {
	icon: typeof Sparkles;
	title: string;
	copy: string;
}) {
	return (
		<div className='rounded-lg border border-(--color-border) bg-(--color-background) p-4'>
			<Icon className='mb-4 size-5 text-(--color-active-text)' />
			<p className='text-sm font-medium'>{title}</p>
			<p className='mt-1 text-xs leading-5 text-(--color-secondary-text)'>{copy}</p>
		</div>
	);
}

function ChoiceButton({
	isSelected,
	onClick,
	title,
	copy,
}: {
	isSelected: boolean;
	onClick: () => void;
	title: string;
	copy: string;
}) {
	return (
		<button
			type='button'
			onClick={onClick}
			className={cn(
				'flex min-h-24 items-start justify-between gap-4 rounded-lg border border-(--color-border) bg-(--color-background) p-4 text-left transition hover:border-(--color-active-text)',
				isSelected && 'border-(--color-active-text) bg-(--color-background-secondary)',
			)}
		>
			<span>
				<span className='block text-sm font-medium'>{title}</span>
				<span className='mt-2 block text-xs leading-5 text-(--color-secondary-text)'>{copy}</span>
			</span>
			<span
				className={cn(
					'grid size-6 shrink-0 place-items-center rounded-full border border-(--color-border)',
					isSelected && 'border-(--color-active-text) bg-(--color-active-text) text-white',
				)}
			>
				{isSelected && <CheckCircle2 className='size-4' />}
			</span>
		</button>
	);
}
