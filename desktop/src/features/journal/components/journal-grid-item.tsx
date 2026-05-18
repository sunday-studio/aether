import { useQueryClient } from '@tanstack/react-query';
import { formatDistanceToNow } from 'date-fns';
import { MoreVertical } from 'lucide-react';
import { useState } from 'react';
import { useDeleteEntry, useUpdateEntry } from '~/aether-sdk';
import { invalidateEntryQueries } from '../invalidate-entry-queries';
import { showToast } from '~/components/shared/toast-components';
import type { EntryWithTags } from '~/types/models';
import { extractFirstSentence } from '../journal.domain.ts';
// import { EntryAudio } from "./entry-audio";
import { JournalActionsDropdown } from './journal-actions-dropdown';
import { JournalAiInsights } from './journal-ai-insights';
import { JournalEditor } from './journal-editor';

interface JournalGridItemProps {
	entry: EntryWithTags;
}

const isEntryDocumentDifferent = (oldDocument: string, newDocument: string) => {
	return oldDocument !== newDocument;
};

export const JournalGridItem = ({ entry }: JournalGridItemProps) => {
	const { mutate: updateEntry } = useUpdateEntry();
	const { mutate: deleteEntry } = useDeleteEntry();
	const queryClient = useQueryClient();

	const [isActionsDropdownOpen, setIsActionsDropdownOpen] = useState(false);
	const [isExpanded, setIsExpanded] = useState(false);

	const title = extractFirstSentence(entry.document ?? '');

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
					showToast({
						title: 'Entry deleted successfully',
					});
				},
			},
		);
	};

	return (
		<div className='flex cursor-pointer flex-col gap-2 rounded-lg border border-neutral-200 bg-white p-4 transition-shadow hover:shadow-md'>
			{/* Header with title and actions */}
			<div className='flex items-start justify-between gap-2'>
				<h3
					className='line-clamp-2 flex-1 cursor-pointer font-medium text-neutral-900'
					onClick={() => setIsExpanded(!isExpanded)}
				>
					{title}
				</h3>
				<JournalActionsDropdown
					entry={entry}
					onDeleteEntry={() => onDeleteEntry(entry.id ?? '')}
					isOpen={isActionsDropdownOpen}
					onOpenChange={setIsActionsDropdownOpen}
				>
					<button
						type='button'
						className='p-1 text-neutral-400 hover:text-neutral-600'
						onClick={e => {
							e.stopPropagation();
							setIsActionsDropdownOpen(true);
						}}
					>
						<MoreVertical className='h-4 w-4' />
					</button>
				</JournalActionsDropdown>
			</div>

			{/* Metadata */}
			<div className='flex items-center gap-2 text-xs text-neutral-500'>
				<span>
					{formatDistanceToNow(new Date(entry.createdAt ?? ''), {
						addSuffix: true,
					})}
				</span>
			</div>

			{/* Audio is intentionally hidden for v1 until the flow is ready to ship. */}
			{/* <EntryAudio entryId={entry.id ?? ""} /> */}

			{/* Expanded editor */}
			{isExpanded && (
				<div className='mt-2 border-t border-neutral-100 pt-2'>
					<JournalEditor
						isSelected={isActionsDropdownOpen}
						document={entry.document ?? ''}
						id={entry.id ?? ''}
						onChange={(document: string) => onUpdateEntry(entry.id ?? '', document)}
					/>
					<JournalAiInsights entryId={entry.id ?? ''} />
				</div>
			)}
		</div>
	);
};
