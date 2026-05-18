import { useQueryClient } from '@tanstack/react-query';
import { CheckCircle2, KeyRound, Sparkles, UserRound } from 'lucide-react';
import { type FormEvent, type ReactNode, useState } from 'react';
import {
	getGetAllSettingsQueryKey,
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
const DEFAULT_PROVIDER_KEY = 'transcription.default_provider';
const OPENAI_API_KEY = 'transcription.openai.api_key';
const GROQ_API_KEY = 'transcription.groq.api_key';

type ProviderChoice = 'openai' | 'groq';

interface OnboardingGateProps {
	children: ReactNode;
}

const providerCopy: Record<
	ProviderChoice,
	{ label: string; description: string; keyLabel: string; placeholder: string }
> = {
	openai: {
		label: 'OpenAI Whisper',
		description: 'Good default if you already use OpenAI.',
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

export function OnboardingGate({ children }: OnboardingGateProps) {
	const queryClient = useQueryClient();
	const [displayName, setDisplayName] = useState('');
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

	if (isLoading) {
		return (
			<div className='grid h-screen w-screen place-items-center bg-(--color-background)'>
				<div className='text-sm text-(--color-secondary-text)'>Loading Aether...</div>
			</div>
		);
	}

	if (isComplete) return <>{children}</>;

	const activeApiKey = provider === 'openai' ? openaiKey : groqKey;
	const hasAnyApiKey = isConfigured(openaiKey) || isConfigured(groqKey);
	const providerStatuses = new Map(
		(providersResponse?.data ?? []).map(item => [item.name, item.status]),
	);

	const completeOnboarding = async (shouldValidateProvider: boolean) => {
		setIsSaving(true);
		setErrorMessage(null);
		setStatusMessage(null);

		try {
			const trimmedName = displayName.trim();
			if (trimmedName) {
				await saveSetting(DISPLAY_NAME_KEY, trimmedName);
			}
			if (isConfigured(openaiKey)) {
				await saveSetting(OPENAI_API_KEY, openaiKey.trim());
			}
			if (isConfigured(groqKey)) {
				await saveSetting(GROQ_API_KEY, groqKey.trim());
			}
			if (hasAnyApiKey) {
				await saveSetting(DEFAULT_PROVIDER_KEY, provider);
			}
			if (shouldValidateProvider && hasAnyApiKey) {
				await validateProvider({
					body: JSON.stringify({ provider_name: provider }),
					headers: { 'Content-Type': 'application/json' },
				});
			}
			await saveSetting(ONBOARDING_COMPLETED_KEY, 'true');
			await queryClient.invalidateQueries({
				queryKey: getGetAllSettingsQueryKey(),
			});
			await queryClient.invalidateQueries({ queryKey: getListProvidersQueryKey() });
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
		completeOnboarding(false);
	};

	return (
		<div className='min-h-screen w-screen overflow-y-auto bg-[radial-gradient(circle_at_top_left,var(--color-card),transparent_35%),linear-gradient(135deg,var(--color-background),var(--color-background-secondary))] px-6 py-10 text-(--color-foreground)'>
			<div className='mx-auto grid min-h-[calc(100vh-5rem)] max-w-6xl grid-cols-1 items-center gap-10 lg:grid-cols-[0.95fr_1.05fr]'>
				<section className='space-y-8'>
					<div className='inline-flex items-center gap-2 rounded-full border border-(--color-border) bg-(--color-card) px-3 py-1 text-xs text-(--color-secondary-text)'>
						<Sparkles className='size-3.5' />
						First launch setup
					</div>
					<div className='space-y-5'>
						<h1 className='newsreader-font max-w-xl text-5xl leading-[0.95] tracking-[-0.04em] md:text-7xl'>
							Make Aether yours before the noise gets in.
						</h1>
						<p className='max-w-lg text-sm leading-6 text-(--color-secondary-text)'>
							Set a name, optionally connect transcription providers, and then drop straight into
							the journal. AI keys are stored through the encrypted settings path.
						</p>
					</div>
					<div className='grid max-w-xl gap-3 text-sm md:grid-cols-3'>
						<div className='rounded-2xl border border-(--color-border) bg-(--color-card)/80 p-4'>
							<UserRound className='mb-5 size-5 text-(--color-active-text)' />
							<p className='font-medium'>Profile</p>
							<p className='mt-1 text-xs text-(--color-secondary-text)'>
								Just enough identity to make the app feel personal.
							</p>
						</div>
						<div className='rounded-2xl border border-(--color-border) bg-(--color-card)/80 p-4'>
							<KeyRound className='mb-5 size-5 text-(--color-active-text)' />
							<p className='font-medium'>AI keys</p>
							<p className='mt-1 text-xs text-(--color-secondary-text)'>
								Optional. Skip now and configure later in Settings.
							</p>
						</div>
						<div className='rounded-2xl border border-(--color-border) bg-(--color-card)/80 p-4'>
							<CheckCircle2 className='mb-5 size-5 text-(--color-active-text)' />
							<p className='font-medium'>Ready path</p>
							<p className='mt-1 text-xs text-(--color-secondary-text)'>
								No placeholder routes, no setup rabbit hole.
							</p>
						</div>
					</div>
				</section>

				<form
					onSubmit={onSubmit}
					className='rounded-[2rem] border border-(--color-border) bg-(--color-card) p-5 shadow-2xl shadow-black/10 md:p-7'
				>
					<div className='space-y-7'>
						<div>
							<p className='text-xs tracking-[0.2em] text-(--color-secondary-text) uppercase'>
								Step 1
							</p>
							<h2 className='mt-2 text-2xl font-medium'>Who is this for?</h2>
							<p className='mt-2 text-sm text-(--color-secondary-text)'>
								The display name is local app context. It is not required.
							</p>
						</div>

						<TextField
							label='Display name'
							placeholder='Ada'
							value={displayName}
							onChange={value => setDisplayName(value)}
						/>

						<div className='space-y-3'>
							<div>
								<p className='text-xs tracking-[0.2em] text-(--color-secondary-text) uppercase'>
									Step 2
								</p>
								<h2 className='mt-2 text-2xl font-medium'>AI transcription</h2>
								<p className='mt-2 text-sm text-(--color-secondary-text)'>
									Connect a provider now, or skip it and use Aether without AI until you are ready.
								</p>
							</div>

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
												'rounded-2xl border border-(--color-border) bg-(--color-background) p-4 text-left transition',
												{
													'border-(--color-active-text) bg-(--color-background-secondary)':
														isSelected,
												},
											)}
										>
											<div className='flex items-center justify-between gap-3'>
												<p className='text-sm font-medium'>{providerCopy[choice].label}</p>
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

							<div className='grid gap-3 md:grid-cols-2'>
								<TextField
									label='OpenAI API Key'
									placeholder={providerCopy.openai.placeholder}
									type='password'
									value={openaiKey}
									onChange={value => setOpenaiKey(value)}
								/>
								<TextField
									label='Groq API Key'
									placeholder={providerCopy.groq.placeholder}
									type='password'
									value={groqKey}
									onChange={value => setGroqKey(value)}
								/>
							</div>
						</div>

						{statusMessage && (
							<div className='rounded-2xl border border-emerald-500/30 bg-emerald-500/10 p-3 text-sm text-emerald-700'>
								{statusMessage}
							</div>
						)}
						{errorMessage && (
							<div className='rounded-2xl border border-red-500/30 bg-red-500/10 p-3 text-sm text-red-600'>
								{errorMessage}
							</div>
						)}

						<div className='flex flex-wrap items-center justify-between gap-3 border-t border-(--color-border) pt-5'>
							<button
								type='button'
								onClick={() => completeOnboarding(false)}
								disabled={isSaving}
								className='text-sm text-(--color-secondary-text) transition hover:text-(--color-active-text) disabled:opacity-50'
							>
								Skip AI setup
							</button>
							<div className='flex items-center gap-3'>
								<Button
									onClick={validateSelectedProvider}
									label={isSaving ? 'Checking...' : 'Validate provider'}
									tooltipContent='Validate the selected provider credentials'
									isDisabled={isSaving || !isConfigured(activeApiKey)}
								/>
								<Button
									onClick={() => completeOnboarding(true)}
									label={isSaving ? 'Saving...' : 'Start using Aether'}
									tooltipContent='Finish onboarding'
									isDisabled={isSaving}
								/>
							</div>
						</div>
					</div>
				</form>
			</div>
		</div>
	);
}
