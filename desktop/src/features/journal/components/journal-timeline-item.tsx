import { useQueryClient } from '@tanstack/react-query';
import { format, formatDistanceToNow } from 'date-fns';
import { useState } from 'react';
import { useDeleteEntry, useUpdateEntry } from '~/aether-sdk';
import { Timeline } from '~/components/shared/timeline';
import { Tooltip } from '~/components/shared/tooltip';
import type { EntryWithTags } from '~/types/models';
import { invalidateEntryQueries } from '../invalidate-entry-queries';
import { JournalActionsDropdown } from './journal-actions-dropdown';
import { JournalEditor } from './journal-editor';
import { EntryTags } from './journal-tags';
import { TaskActionButton } from '~/features/tasks/components/task-item/task-shared-components';
import { Trash } from 'lucide-react';
import { Button } from 'react-aria-components';

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
		<div className='mx-auto w-3xl'>
			<JournalEditor
				isSelected={isActionsDropdownOpen}
				document={entry.document ?? ''}
				id={entry.id ?? ''}
				onChange={(document: string) => onUpdateEntry(entry.id ?? '', document)}
			/>
			<div className='flex items-center justify-between gap-2'>
				<EntryTags entry={entry} />
				<div className='flex items-center gap-1.5'>
					<div className='group relative ml-auto w-fit shrink-0'>
						<Tooltip
							trigger={
								<p className='font-gt-ultra cursor-default rounded-md px-1 py-0.5 text-right text-xs text-neutral-400 italic'>
									{formatDistanceToNow(new Date(entry.createdAt ?? ''), {
										addSuffix: true,
									})}
								</p>
							}
							content={`created at ${format(new Date(), 'MMMM d, yyyy')}`}
						/>
					</div>

					<Button onPress={() => onDeleteEntry(entry.id ?? '')}>
						<TaskActionButton>
							<Trash size={12} strokeWidth={3} />
						</TaskActionButton>
					</Button>
				</div>
			</div>
		</div>
	);
};
