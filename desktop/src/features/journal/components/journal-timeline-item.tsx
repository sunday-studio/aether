import { useQueryClient } from '@tanstack/react-query';
import { format, formatDistanceToNow } from 'date-fns';
import { useState } from 'react';
import { useDeleteEntry, useUpdateEntry } from '~/aether-sdk';
import { Timeline } from '~/components/shared/timeline';
import { Tooltip } from '~/components/shared/tooltip';
import type { EntryWithTags } from '~/types/models';
import { invalidateEntryQueries } from '../invalidate-entry-queries';
// import { EntryAudio } from './entry-audio';
import { JournalActionsDropdown } from './journal-actions-dropdown';
import { JournalEditor } from './journal-editor';
import { EntryTags } from './journal-tags';
// import { JournalAiInsights } from './journal-ai-insights';

interface JournalTimelineItemProps {
	entry: EntryWithTags;
}

const isEntryDocumentDifferent = (oldDocument: string, newDocument: string) => {
	return oldDocument !== newDocument;
};

export const JournalTimelineItem = ({ entry }: JournalTimelineItemProps) => {
	const { mutate: updateEntry } = useUpdateEntry();
	const { mutate: deleteEntry } = useDeleteEntry();

	const queryClient = useQueryClient();

	const [isActionsDropdownOpen, setIsActionsDropdownOpen] = useState(false);

	const onUpdateEntry = async (entryId: string, document: string) => {
		if (isEntryDocumentDifferent(entry.document ?? '', document)) {
			updateEntry(
				{
					id: entryId,
					data: {
						document,
					},
				},
				{
					onSuccess: () => invalidateEntryQueries(queryClient),
				},
			);
		}
	};

	const onDeleteEntry = async (entryId: string) => {
		deleteEntry(
			{
				id: entryId,
			},
			{
				onSuccess: () => {
					invalidateEntryQueries(queryClient);
				},
			},
		);
	};

	return (
		<Timeline.Item
			key={entry.id}
			className='bg-red-0 w-3xl'
			indicatorContainerClassName='w-10'
			leftContainerClassName='w-50'
			indicator={
				<JournalActionsDropdown
					entry={entry}
					onDeleteEntry={() => onDeleteEntry(entry.id ?? '')}
					isOpen={isActionsDropdownOpen}
					onOpenChange={setIsActionsDropdownOpen}
				>
					<Timeline.Indicator
						className='cursor-pointer'
						containerClassName='col-end-9 col-start-10'
						onClick={() => setIsActionsDropdownOpen(true)}
					/>
				</JournalActionsDropdown>
			}
			leftContent={
				<Timeline.LeftContent className='flex flex-col items-end gap-1'>
					<div className='group relative ml-auto w-fit shrink-0'>
						<Tooltip
							trigger={
								<p className='font-gt-ultra cursor-default rounded-md px-1 py-0.5 text-right text-xs text-neutral-500'>
									{formatDistanceToNow(new Date(entry.createdAt ?? ''), {
										addSuffix: true,
									})}
								</p>
							}
							content={`created at ${format(new Date(), 'MMMM d, yyyy')}`}
						/>
					</div>
				</Timeline.LeftContent>
			}
			rightContent={
				<Timeline.RightContent className='mb-5 flex flex-col gap-3'>
					{/* Audio is intentionally hidden for v1 until the flow is ready to ship. */}
					{/* <EntryAudio entryId={entry.id ?? ''} /> */}
					<JournalEditor
						isSelected={isActionsDropdownOpen}
						document={entry.document ?? ''}
						id={entry.id ?? ''}
						onChange={(document: string) => onUpdateEntry(entry.id ?? '', document)}
					/>

					<EntryTags entry={entry} />
					{/* AI journal insights are hidden for now. */}
					{/* <JournalAiInsights entryId={entry.id ?? ''} /> */}
				</Timeline.RightContent>
			}
		></Timeline.Item>
	);
};
