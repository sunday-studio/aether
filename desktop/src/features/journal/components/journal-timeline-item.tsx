import { useQueryClient } from "@tanstack/react-query";
import { format, formatDistanceToNow } from "date-fns";
import { useState } from "react";
import {
	getGetEntriesQueryKey,
	useDeleteEntry,
	useUpdateEntry,
} from "~/aether-sdk";
import type { DbEntry } from "~/aether-sdk/models";
import { Timeline } from "~/components/shared/timeline";
import { showToast } from "~/components/shared/toast-components";
import { Tooltip } from "~/components/shared/tooltip";
import { JournalActionsDropdown } from "./journal-actions-dropdown";
import { JournalEditor } from "./journal-editor";
import { EntryTags } from "./journal-tags";

interface JournalTimelineItemProps {
	entry: DbEntry;
}

export const JournalTimelineItem = ({ entry }: JournalTimelineItemProps) => {
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
		<Timeline.Item
			key={entry.id}
			className="grid-cols-24 grid"
			leftContainerClassName="col-start-5 col-end-9"
			rightContainerClassName="col-start-10 col-end-20"
			indicator={
				<JournalActionsDropdown
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
						containerClassName="col-end-9 col-start-10"
						onClick={() => setIsActionsDropdownOpen(true)}
					/>
				</JournalActionsDropdown>
			}
			leftContent={
				<Timeline.LeftContent className="flex flex-col gap-1 items-end">
					<div className="relative group w-fit ml-auto shrink-0">
						<Tooltip
							trigger={
								<p className="text-xs text-neutral-500 text-right newsreader-font px-1 py-0.5 rounded-md cursor-default">
									{formatDistanceToNow(new Date(entry.createdAt ?? ""), {
										addSuffix: true,
									})}
								</p>
							}
							content={`created at ${format(new Date(), "MMMM d, yyyy")}`}
						/>
					</div>

					{shouldShowTags && <EntryTags entry={entry} />}
				</Timeline.LeftContent>
			}
			rightContent={
				<Timeline.RightContent className="mb-5 flex flex-col gap-1">
					<JournalEditor
						isSelected={isActionsDropdownOpen}
						document={entry.document ?? ""}
						id={entry.id ?? ""}
						onChange={(document: string) =>
							onUpdateEntry(entry.id ?? "", document)
						}
					/>
				</Timeline.RightContent>
			}
		></Timeline.Item>
	);
};

// TODOs:
// - add infinite scrolling; fetch first 200 entries
