import { Download, HardDrive } from 'lucide-react';
import { Button } from '~/components/shared/button';
import { formatModelSize } from '~/lib/search-embedding-models';
import { type EmbeddingModelSummary, type SearchChoice } from '../onboarding.types';
import { ChoiceButton } from './choice-button';

interface SearchStepProps {
	searchChoice: SearchChoice;
	defaultEmbeddingModel?: EmbeddingModelSummary;
	isStartingModelDownload: boolean;
	onSearchChoiceChange: (value: SearchChoice) => void;
	onStartEmbeddingModelDownload: () => void;
}

export function SearchStep({
	searchChoice,
	defaultEmbeddingModel,
	isStartingModelDownload,
	onSearchChoiceChange,
	onStartEmbeddingModelDownload,
}: SearchStepProps) {
	return (
		<div className='space-y-6'>
			<div>
				<h3 className='text-3xl font-medium'>Set up local search?</h3>
				<p className='mt-3 max-w-xl text-sm leading-6 text-(--color-secondary-text)'>
					Local search can index journal entries and tasks on this device. You can skip it and
					configure the model later in Settings.
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
							Download a local model for semantic search. It runs on this device, is about{' '}
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
					isSelected={searchChoice === 'yes'}
					onClick={() => onSearchChoiceChange('yes')}
					title='Set up search'
					copy='Use a local model for journal and task search.'
				/>
				<ChoiceButton
					isSelected={searchChoice === 'no'}
					onClick={() => onSearchChoiceChange('no')}
					title='Skip for now'
					copy='Use keyword search and configure the model later.'
				/>
			</div>
		</div>
	);
}
