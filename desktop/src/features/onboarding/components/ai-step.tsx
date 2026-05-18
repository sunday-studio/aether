import { Download, HardDrive } from 'lucide-react';
import { motion } from 'motion/react';
import { Button } from '~/components/shared/button';
import { TextField } from '~/components/shared/text-field';
import { formatModelSize } from '~/lib/search-embedding-models';
import { cn } from '~/utils/cn';
import { providerCopy } from '../onboarding.constants';
import {
	type AiChoice,
	type EmbeddingModelSummary,
	type ProviderChoice,
} from '../onboarding.types';
import { ChoiceButton } from './choice-button';

interface AiStepProps {
	aiChoice: AiChoice;
	provider: ProviderChoice;
	activeApiKey: string;
	providerStatuses: Map<string, string>;
	defaultEmbeddingModel?: EmbeddingModelSummary;
	isStartingModelDownload: boolean;
	onAiChoiceChange: (value: AiChoice) => void;
	onProviderChange: (value: ProviderChoice) => void;
	onApiKeyChange: (value: string) => void;
	onStartEmbeddingModelDownload: () => void;
}

export function AiStep({
	aiChoice,
	provider,
	activeApiKey,
	providerStatuses,
	defaultEmbeddingModel,
	isStartingModelDownload,
	onAiChoiceChange,
	onProviderChange,
	onApiKeyChange,
	onStartEmbeddingModelDownload,
}: AiStepProps) {
	return (
		<div className='space-y-6'>
			<div>
				<h3 className='text-3xl font-medium'>Use AI in Aether?</h3>
				<p className='mt-3 max-w-xl text-sm leading-6 text-(--color-secondary-text)'>
					AI configuration is optional. Local search intelligence can run on this device, and hosted
					transcription only works when you add a provider key.
				</p>
			</div>
			<div className='rounded-lg border border-(--color-border) bg-(--color-background) p-4'>
				<div className='flex flex-wrap items-start justify-between gap-4'>
					<div className='min-w-0 flex-1'>
						<div className='flex items-center gap-2'>
							<HardDrive className='size-4 text-(--color-active-text)' />
							<p className='text-sm font-medium'>Offline search model</p>
						</div>
						<p className='mt-2 text-xs leading-5 text-(--color-secondary-text)'>
							Download a local model for related notes and semantic search. It runs on this device,
							is about{' '}
							{defaultEmbeddingModel ? formatModelSize(defaultEmbeddingModel.fileSize) : '100 MB'},
							and can be skipped.
						</p>
						{defaultEmbeddingModel?.isDownloaded && (
							<p className='mt-2 truncate text-xs text-(--color-secondary-text)'>
								Downloaded at {defaultEmbeddingModel.modelPath}
							</p>
						)}
					</div>
					<Button
						onClick={onStartEmbeddingModelDownload}
						label={
							defaultEmbeddingModel?.isDownloaded
								? 'Downloaded'
								: isStartingModelDownload
									? 'Starting...'
									: 'Download now'
						}
						tooltipContent='Download the local search model'
						isDisabled={isStartingModelDownload || Boolean(defaultEmbeddingModel?.isDownloaded)}
						iconLeft={<Download className='size-4' />}
					/>
				</div>
			</div>
			<div className='grid gap-3 md:grid-cols-2'>
				<ChoiceButton
					isSelected={aiChoice === 'yes'}
					onClick={() => onAiChoiceChange('yes')}
					title='Yes, enable AI'
					copy='Choose a provider and save a key.'
				/>
				<ChoiceButton
					isSelected={aiChoice === 'no'}
					onClick={() => onAiChoiceChange('no')}
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
									onClick={() => onProviderChange(choice)}
									className={cn(
										'rounded-lg border border-(--color-border) bg-(--color-background) p-4 text-left transition hover:border-(--color-active-text)',
										isSelected && 'border-(--color-active-text) bg-(--color-background-secondary)',
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
					<TextField
						label={providerCopy[provider].keyLabel}
						placeholder={providerCopy[provider].placeholder}
						type='password'
						value={activeApiKey}
						onChange={onApiKeyChange}
					/>
				</motion.div>
			)}
		</div>
	);
}
