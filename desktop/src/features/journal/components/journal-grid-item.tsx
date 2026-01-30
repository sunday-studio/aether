import { formatDistanceToNow } from "date-fns";
import { useState } from "react";
import { MoreVertical } from "lucide-react";
import type { EntryWithTags } from "~/types/models";
import { extractFirstSentence } from "../journal.domain.ts";
import { JournalEditor } from "./journal-editor";
import { JournalActionsDropdown } from "./journal-actions-dropdown";
import {
	getGetEntriesQueryKey,
	useDeleteEntry,
	useUpdateEntry,
} from "~/aether-sdk";
import { useQueryClient } from "@tanstack/react-query";
import { showToast } from "~/components/shared/toast-components";
import { EntryAudio } from "./entry-audio";

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
	const entriesQueryKey = getGetEntriesQueryKey();

	const [isActionsDropdownOpen, setIsActionsDropdownOpen] = useState(false);
	const [isExpanded, setIsExpanded] = useState(false);

	const title = extractFirstSentence(entry.document ?? "");

	const onUpdateEntry = async (entryId: string, document: string) => {
		if (isEntryDocumentDifferent(entry.document ?? "", document)) {
			updateEntry(
				{
					id: entryId,
					data: {
						document,
					},
				},
				{
					onSuccess: () => {
						queryClient.invalidateQueries({ queryKey: entriesQueryKey });
					},
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
					queryClient.invalidateQueries({ queryKey: entriesQueryKey });
					showToast({
						title: "Entry deleted successfully",
					});
				},
			},
		);
	};

	return (
		<div className="bg-white rounded-lg border border-neutral-200 p-4 hover:shadow-md transition-shadow cursor-pointer flex flex-col gap-2">
			{/* Header with title and actions */}
			<div className="flex items-start justify-between gap-2">
				<h3
					className="font-medium text-neutral-900 flex-1 line-clamp-2 cursor-pointer"
					onClick={() => setIsExpanded(!isExpanded)}
				>
					{title}
				</h3>
				<JournalActionsDropdown
					entry={entry}
					onDeleteEntry={() => onDeleteEntry(entry.id ?? "")}
					onPinEntry={() => {}}
					onArchiveEntry={() => {}}
					isOpen={isActionsDropdownOpen}
					onOpenChange={setIsActionsDropdownOpen}
					onAddTags={() => {}}
				>
					<button
						type="button"
						className="text-neutral-400 hover:text-neutral-600 p-1"
						onClick={(e) => {
							e.stopPropagation();
							setIsActionsDropdownOpen(true);
						}}
					>
						<MoreVertical className="w-4 h-4" />
					</button>
				</JournalActionsDropdown>
			</div>

			{/* Metadata */}
			<div className="flex items-center gap-2 text-xs text-neutral-500">
				<span>{formatDistanceToNow(new Date(entry.createdAt ?? ""), { addSuffix: true })}</span>
			</div>

			{/* Audio if available */}
			<EntryAudio entryId={entry.id ?? ""} />

			{/* Expanded editor */}
			{isExpanded && (
				<div className="mt-2 border-t border-neutral-100 pt-2">
					<JournalEditor
						isSelected={isActionsDropdownOpen}
						document={entry.document ?? ""}
						id={entry.id ?? ""}
						onChange={(document: string) =>
							onUpdateEntry(entry.id ?? "", document)
						}
					/>
				</div>
			)}
		</div>
	);
};
