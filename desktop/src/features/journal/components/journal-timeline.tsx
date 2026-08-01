import { Loader } from 'lucide-react';
import { Button } from '~/components/shared/button';
import { useCreateJournalEntry } from '~/hooks/use-create-journal-entry.ts';
import { useEntriesInfinite } from '~/hooks/use-entries-infinite';
import { sortEntries } from '../journal.domain.ts';
import { JournalTimelineItem } from './journal-timeline-item.tsx';

export const JournalTimeline = () => {
	const { items, sentinelRef, isFetchingMore } = useEntriesInfinite();
	const { createEntry } = useCreateJournalEntry();

	const sortedEntries = sortEntries(items);

	const hasNoEntries = sortedEntries.length === 0;

	if (hasNoEntries) {
		return (
			<div className='relative flex h-full items-center justify-center'>
				<div className='flex flex-col items-center text-sm text-neutral-500'>
					<p className=''>
						First day? Welcome to <span className='text-neutral-800'>Aether</span>.
					</p>
					<p className='mb-6'>Let's start with a new journal entry.</p>
					<Button
						onClick={createEntry}
						label="Let's start"
						shortcuts={['⌘', 'N']}
						tooltipContent='Create the first one'
					/>
				</div>
			</div>
		);
	}

	return (
		<div className='relative mt-2 mb-200! flex h-full justify-center overflow-y-scroll'>
			<div className='bg-red-0 w-full max-w-5xl space-y-6 pt-6'>
				<div className='mx-auto w-3xl'>
					<Button
						onClick={createEntry}
						label='Write'
						shortcuts={['⌘', 'N']}
						tooltipContent='Create a new entry'
					/>
				</div>

				{sortedEntries?.map(entry => {
					return <JournalTimelineItem key={entry.id} entry={entry} />;
				})}
			</div>
			<div ref={sentinelRef} className='flex justify-center py-8'>
				{isFetchingMore && <Loader className='h-4 w-4 animate-spin text-neutral-400' />}
			</div>
		</div>
	);
};
