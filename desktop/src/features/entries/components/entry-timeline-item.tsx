import { useQueryClient } from "@tanstack/react-query";
import clsx from "clsx";
import { format, isToday as isTodayFn } from "date-fns";
import { useState } from "react";
import {
	getGetEntryQueryKey,
	useCreateEntry,
	useUpdateEntry,
} from "~/aether-sdk";
import type { DbEntry } from "~/aether-sdk/models";
import { Timeline } from "~/components/shared/timeline";
import { EntryActionsDropdown } from "./entry-actions-dropdown";
import { EntryEditor } from "./entry-editor";
import { EntryTags } from "./entry-tags";

interface EntryTimelineItemProps {
	entry: DbEntry;
}

const placeholder =
	'{"root":{"children":[{"children":[],"direction":"ltr","format":"","indent":0,"type":"paragraph","version":1,"textFormat":0,"textStyle":""}],"direction":"ltr","format":"","indent":0,"type":"root","version":1}}';

export const AddNewEntryButton = ({ onClick }: { onClick: () => void }) => {
	return (
		<button
			className={clsx(
				"bg-neutral-200",
				"text-neutral-700",
				"px-3",
				"py-1",
				"rounded-full",
				"text-sm",
				"hover:ring-neutral-300",
				"ring-3",
				"ring-transparent",
				"transition",
				"duration-200",
				"cursor-pointer",
			)}
			type="button"
			onClick={onClick}
		>
			Write
		</button>
	);
};

export const EntryTimelineItem = ({ entry }: EntryTimelineItemProps) => {
	const [isActionsDropdownOpen, setIsActionsDropdownOpen] = useState(false);
	const [isTagsShown, setIsTagsShown] = useState(false);

	const shouldShowTags =
		isTagsShown || (entry?.tags && entry?.tags?.length > 0);

	console.log("shouldShowTags", {
		shouldShowTags,
		isTagsShown,
		entryTags: entry.tags,
	});

	return (
		<Timeline.Item key={entry.id}>
			<EntryActionsDropdown
				entry={entry}
				isOpen={isActionsDropdownOpen}
				onOpenChange={setIsActionsDropdownOpen}
				onAddTags={() => {
					console.log("add tags");
					setIsTagsShown(true);
				}}
			>
				<Timeline.Indicator
					className="cursor-pointer"
					onClick={() => setIsActionsDropdownOpen(true)}
				/>
			</EntryActionsDropdown>
			<Timeline.Content className="mb-5 flex flex-col ">
				{shouldShowTags && <EntryTags entry={entry} />}
				<EntryEditor
					isSelected={isActionsDropdownOpen}
					createdAt={entry.createdAt ?? ""}
					updatedAt={entry.updatedAt ?? ""}
					document={entry.document ?? ""}
					id={entry.id ?? ""}
					onChange={() => {}}
					// onChange={(document) =>
					// 	onUpdateEntry(entry.id ?? "", document)
					// }
				/>
			</Timeline.Content>
		</Timeline.Item>
	);
};

// TODOs:
// - add infinite scrolling; fetch first 200 entries

// const queryKey = getGetEntryQueryKey();
// const queryClient = useQueryClient();

// const [entries, setEntries] = useState<DbEntry[]>(data);
// const isToday = isTodayFn(date);
// const hasEntries = entries.length > 0;

// const { mutate: createEntry } = useCreateEntry();
// const { mutate: updateEntry } = useUpdateEntry();

// const onUpdateEntry = async (entryId: string, document: string) => {
// 	updateEntry(
// 		{
// 			id: entryId,
// 			data: {
// 				document,
// 			},
// 		},
// 		{
// 			onSuccess: () => {
// 				queryClient.invalidateQueries({ queryKey });
// 			},
// 		},
// 	);
// };

// const onAddNewEntry = async () => {
// 	const now = new Date();

// 	createEntry(
// 		{
// 			data: {
// 				document: placeholder,
// 				date: now.toISOString(),
// 			},
// 		},
// 		{
// 			onSuccess: (data) => {
// 				setEntries([...entries, data.data]);
// 				queryClient.invalidateQueries({ queryKey });
// 			},
// 			onError: (error) => {
// 				console.log("error ->", error);
// 				console.error(error);
// 			},
// 		},
// 	);
// };
