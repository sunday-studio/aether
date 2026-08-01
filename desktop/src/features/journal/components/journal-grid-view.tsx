import { useGetEntries } from '~/aether-sdk';
import { Button } from '~/components/shared/button';
import { useCreateJournalEntry } from '~/hooks/use-create-journal-entry.ts';
import { groupEntriesByTags, sortEntries } from '../journal.domain.ts';
import { JournalGridItem } from './journal-grid-item.tsx';

export const JournalGridView = () => {
	const { data: entriesResponse } = useGetEntries();
	const { createEntry } = useCreateJournalEntry();

	// SDK now returns properly typed PaginatedEntries
	const sortedEntries = sortEntries(entriesResponse?.data?.items ?? []);

	const groupedByTags = groupEntriesByTags(sortedEntries);

	return (
		<div className='h-full overflow-y-scroll bg-neutral-50'>
			{/* Header with actions */}
			<div className='sticky top-0 z-10 border-b border-neutral-200 bg-neutral-50 px-6 py-4'>
				<div className='mx-auto flex max-w-7xl items-center gap-2'>
					<Button
						onClick={createEntry}
						label='Write'
						shortcuts={['⌘', 'N']}
						tooltipContent='Create a new entry'
					/>
				</div>
			</div>

			{/* Grid content grouped by tags */}
			<div className='mx-auto max-w-7xl p-6'>
				{groupedByTags.size === 0 ? (
					<div className='py-12 text-center text-neutral-500'>
						<p>No entries yet. Create your first entry to get started.</p>
					</div>
				) : (
					Array.from(groupedByTags.entries()).map(([tagId, { tagName, entries }]) => (
						<div key={tagId} className='mb-8'>
							<h2 className='mb-4 px-2 text-lg font-semibold text-neutral-900'>{tagName}</h2>
							<div className='grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4'>
								{entries.map(entry => (
									<JournalGridItem key={entry.id} entry={entry} />
								))}
							</div>
						</div>
					))
				)}
			</div>
		</div>
	);
};
