import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
	getGetEntriesQueryKey,
	useDeleteEntry,
	useUpdateEntry,
} from "~/aether-sdk";
import type { DbEntry } from "~/aether-sdk/models";
import { Timeline } from "~/components/shared/timeline";
import { showToast } from "~/components/shared/toast-components";
import { EntryActionsDropdown } from "./entry-actions-dropdown";
import { EntryEditor } from "./entry-editor";
import { EntryTags } from "./entry-tags";

interface EntryTimelineItemProps {
	entry: DbEntry;
}

export const EntryTimelineItem = ({ entry }: EntryTimelineItemProps) => {
	const { mutate: updateEntry } = useUpdateEntry();
	const { mutate: deleteEntry } = useDeleteEntry();

	const queryClient = useQueryClient();
	const entriesQueryKey = getGetEntriesQueryKey();

	const [isActionsDropdownOpen, setIsActionsDropdownOpen] = useState(false);
	const [isTagsShown, setIsTagsShown] = useState(false);

	const shouldShowTags =
		isTagsShown || (entry?.tags && entry?.tags?.length > 0);

	const onUpdateEntry = async (entryId: string, document: string) => {
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
		<Timeline.Item key={entry.id}>
			<EntryActionsDropdown
				entry={entry}
				onDeleteEntry={() => onDeleteEntry(entry.id ?? "")}
				onPinEntry={() => {}}
				onArchiveEntry={() => {}}
				isOpen={isActionsDropdownOpen}
				onOpenChange={setIsActionsDropdownOpen}
				onAddTags={() => {
					setIsTagsShown(true);
				}}
			>
				<Timeline.Indicator
					className="cursor-pointer"
					onClick={() => setIsActionsDropdownOpen(true)}
				/>
			</EntryActionsDropdown>
			<Timeline.Content className="mb-5 flex flex-col gap-1">
				{shouldShowTags && <EntryTags entry={entry} />}
				<EntryEditor
					isSelected={isActionsDropdownOpen}
					createdAt={entry.createdAt ?? ""}
					updatedAt={entry.updatedAt ?? ""}
					document={entry.document ?? ""}
					id={entry.id ?? ""}
					onChange={(document) => onUpdateEntry(entry.id ?? "", document)}
				/>
			</Timeline.Content>
		</Timeline.Item>
	);
};

// TODOs:
// - add infinite scrolling; fetch first 200 entries
