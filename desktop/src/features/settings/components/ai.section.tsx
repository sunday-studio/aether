import { useQueryClient } from '@tanstack/react-query';
import { CheckCircle2, CircleAlert, KeyRound, WandSparkles } from 'lucide-react';
import { useState } from 'react';
import {
	getGetAllSettingsQueryKey,
	getListProvidersQueryKey,
	setSetting,
	useGetAllSettings,
	useListProviders,
	validateProvider,
} from '~/aether-sdk';
import { Button } from '~/components/shared/button';
import { Select, SelectItem } from '~/components/shared/select';
import { TextField } from '~/components/shared/text-field';
import { cn } from '~/utils/cn';

type ProviderChoice = 'openai' | 'groq';

const DEFAULT_PROVIDER_KEY = 'transcription.default_provider';
const OPENAI_API_KEY = 'transcription.openai.api_key';
const GROQ_API_KEY = 'transcription.groq.api_key';

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
	const [statusMessage, setStatusMessage] = useState<string | null>(null);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const { data: settingsResponse } = useGetAllSettings({
		query: { queryKey: getGetAllSettingsQueryKey() },
	});
	const { data: providersResponse } = useListProviders({
		query: { queryKey: getListProvidersQueryKey() },
	});

	const settings = settingsResponse?.data ?? {};
	const savedProvider = (settings[DEFAULT_PROVIDER_KEY] as ProviderChoice | undefined) ?? 'openai';
	const selectedProvider = provider ?? savedProvider;
	const providers = providersResponse?.data ?? [];
	const providerStatuses = new Map(providers.map(item => [item.name, item]));
	const activeProvider = providerStatuses.get(selectedProvider);
	const hasOpenAiKey = isConfigured(settings[OPENAI_API_KEY]);
	const hasGroqKey = isConfigured(settings[GROQ_API_KEY]);
	const activeApiKey = selectedProvider === 'openai' ? openaiKey : groqKey;
	const selectedProviderHasSavedKey = selectedProvider === 'openai' ? hasOpenAiKey : hasGroqKey;

	const invalidateAiQueries = async () => {
		await queryClient.invalidateQueries({ queryKey: getGetAllSettingsQueryKey() });
		await queryClient.invalidateQueries({ queryKey: getListProvidersQueryKey() });
	};

	const saveKeysAndProvider = async () => {
		setIsSaving(true);
		setStatusMessage(null);
		setErrorMessage(null);

		try {
			if (openaiKey.trim()) {
				await setSetting({ key: OPENAI_API_KEY, value: openaiKey.trim() });
			}
			if (groqKey.trim()) {
				await setSetting({ key: GROQ_API_KEY, value: groqKey.trim() });
			}
			await setSetting({ key: DEFAULT_PROVIDER_KEY, value: selectedProvider });
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
				await setSetting({
					key: selectedProvider === 'openai' ? OPENAI_API_KEY : GROQ_API_KEY,
					value: activeApiKey.trim(),
				});
			}
			await setSetting({ key: DEFAULT_PROVIDER_KEY, value: selectedProvider });
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

	return (
		<div className='w-full space-y-8'>
			<div>
				<h3 className='flex items-center gap-2 text-lg font-medium'>
					<WandSparkles className='size-5' />
					AI
				</h3>
				<p className='mt-2 text-sm text-(--color-secondary-text)'>
					AI is optional for v1. Add hosted transcription keys here when you want journal audio
					transcription.
				</p>
			</div>

			<div className='rounded-lg border border-(--color-border) bg-(--color-panel) p-4'>
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
							activeProvider?.status === 'ready'
								? 'border-emerald-500/30 bg-emerald-500/10 text-emerald-700'
								: 'border-(--color-border) text-(--color-secondary-text)',
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
