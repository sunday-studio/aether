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
import { EntryEditor } from "./entry-editor";

interface EntryTimelineItemProps {
	entry: DbEntry["document"];
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

const EmptyState = ({ onAddNewEntry }: { onAddNewEntry: () => void }) => {
	return (
		<Timeline>
			<Timeline.Item>
				<Timeline.Indicator />
				<Timeline.Content>
					<p className="text-neutral-500">No entries for this day</p>
				</Timeline.Content>
			</Timeline.Item>

			<Timeline.Item>
				<Timeline.Indicator />
				<Timeline.Content className="-my-1">
					<AddNewEntryButton onClick={onAddNewEntry} />
				</Timeline.Content>
			</Timeline.Item>
		</Timeline>
	);
};

export const EntryTimelineItem = ({ entry }: EntryTimelineItemProps) => {
	const queryKey = getGetEntryQueryKey();
	const queryClient = useQueryClient();

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

	return (
		<div
		// className={clsx("mb-8 border-b border-neutral-200", {
		// 	"": isToday || data.length > 0,
		// })}
		>
			{/* <div className="sticky top-0 pt-4 pb-2 z-10 text-neutral-700 newsreader-font font-medium px-4 backdrop-blur-lg ">
				{format(date, "EEEE, MMMM d")}
			</div> */}

			{/* todo: tags come here */}

			<div className="space-y-2 mt-4 h-full flex flex-col gap-2 px-4 relative">
				{/* {hasEntries ? (
					<Timeline>
						{entries.map((entry) => (
							<Timeline.Item key={entry.id}>
								<Timeline.Indicator className="cursor-pointer" />
								<Timeline.Content className="mb-0">
									<EntryEditor
										createdAt={entry.createdAt ?? ""}
										updatedAt={entry.updatedAt ?? ""}
										document={entry.document ?? ""}
										id={entry.id ?? ""}
										onChange={(document) =>
											onUpdateEntry(entry.id ?? "", document)
										}
									/>
								</Timeline.Content>
							</Timeline.Item>
						))}
						<Timeline.Item>
							<Timeline.Indicator />
							<Timeline.Content className="-my-1">
								<AddNewEntryButton onClick={onAddNewEntry} />
							</Timeline.Content>
						</Timeline.Item>
					</Timeline>
				) : (
					<EmptyState onAddNewEntry={onAddNewEntry} />
				)} */}
			</div>
		</div>
	);
};
