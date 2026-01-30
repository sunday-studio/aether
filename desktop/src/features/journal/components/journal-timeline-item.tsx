import { useQueryClient } from "@tanstack/react-query";
import { format, formatDistanceToNow } from "date-fns";
import { useState } from "react";
import {
	getGetEntriesInfiniteQueryKey,
	getGetEntriesQueryKey,
	useDeleteEntry,
	useUpdateEntry,
} from "~/aether-sdk";
import { Timeline } from "~/components/shared/timeline";
import { showToast } from "~/components/shared/toast-components";
import { Tooltip } from "~/components/shared/tooltip";
import type { EntryWithTags } from "~/types/models";
// import { EntryAudio } from "./entry-audio";
import { JournalActionsDropdown } from "./journal-actions-dropdown";
import { JournalEditor } from "./journal-editor";

// import { EntryTags } from "./journal-tags";

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
	const entriesQueryKey = getGetEntriesInfiniteQueryKey();

	const [isActionsDropdownOpen, setIsActionsDropdownOpen] = useState(false);
	const [isTagsShown, setIsTagsShown] = useState(false);

	const shouldShowTags =
		isTagsShown || (entry?.tags && entry?.tags?.length > 0);

	const onUpdateEntry = async (entryId: string, document: string) => {
		if (isEntryDocumentDifferent(entry.document ?? "", document)) {
			console.log("onUpdateEntry", entryId, document);
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
		<Timeline.Item
			key={entry.id}
			className="w-3xl bg-red-0"
			indicatorContainerClassName="w-10"
			leftContainerClassName="w-40"
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
								<p className="text-xs text-neutral-500 text-right font-gt-ultra px-1 py-0.5 rounded-md cursor-default">
									{formatDistanceToNow(new Date(entry.createdAt ?? ""), {
										addSuffix: true,
									})}
								</p>
							}
							content={`created at ${format(new Date(), "MMMM d, yyyy")}`}
						/>
					</div>
				</Timeline.LeftContent>
			}
			rightContent={
				<Timeline.RightContent className="mb-5 flex flex-col gap-1 ">
					{/* {shouldShowTags && <EntryTags entry={entry} />} */}
					{/* <EntryAudio entryId={entry.id ?? ""} /> */}
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
