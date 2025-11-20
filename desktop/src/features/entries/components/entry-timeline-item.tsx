import { format, isToday as isTodayFn } from "date-fns";
import clsx from "clsx";
import { EntryEditor } from "./entry-editor";
import type { DbEntry } from "~/aether-sdk/models";
import {
	useCreateEntry,
	useUpdateEntry,
	getGetEntryQueryKey,
} from "~/aether-sdk";
import { Timeline } from "~/components/shared/timeline";
import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";

interface EntryTimelineItemProps {
	date: Date;
	data?: DbEntry[] | [];
}

const placeholder =
	'{"root":{"children":[{"children":[{"detail":0,"format":0,"mode":"normal","style":"","text":"Hello world….","type":"text","version":1}],"direction":"ltr","format":"","indent":0,"type":"paragraph","version":1,"textFormat":0,"textStyle":""}],"direction":"ltr","format":"","indent":0,"type":"root","version":1}}';

const AddNewEntryButton = ({ onClick }: { onClick: () => void }) => {
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

export const EntryTimelineItem = ({
	date,
	data = [],
}: EntryTimelineItemProps) => {
	const queryKey = getGetEntryQueryKey();
	const queryClient = useQueryClient();

	const [entries, setEntries] = useState<DbEntry[]>(data);
	const isToday = isTodayFn(date);
	const hasEntries = entries.length > 0;

	const { mutate: createEntry } = useCreateEntry();
	const { mutate: updateEntry } = useUpdateEntry();

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
					queryClient.invalidateQueries({ queryKey });
				},
			},
		);
	};

	const onAddNewEntry = async () => {
		const now = new Date();

		createEntry(
			{
				data: {
					document: placeholder,
					date: now.toISOString(),
				},
			},
			{
				onSuccess: (data) => {
					setEntries([...entries, data.data]);
					queryClient.invalidateQueries({ queryKey });
				},
				onError: (error) => {
					console.log("error ->", error);
					console.error(error);
				},
			},
		);
	};

	return (
		<div
			className={clsx("mb-8 border-b border-neutral-200", {
				"": isToday || data.length > 0,
				// "h-32 bg-green-50 ring ring-green-400": !isToday,
			})}
		>
			<div className="sticky top-0 pt-4 pb-2 z-10 text-neutral-700 newsreader-font font-medium px-4 backdrop-blur-lg bg-white/20">
				{format(date, "EEEE, MMMM d")}
			</div>

			{/* todo: tags come here */}

			<div className="space-y-2 mt-4 h-full flex flex-col gap-2 px-4 relative">
				{hasEntries ? (
					<Timeline>
						{entries.map((entry) => (
							<Timeline.Item key={entry.id}>
								<Timeline.Indicator />
								<Timeline.Content>
									<EntryEditor
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
				)}
			</div>
		</div>
	);
};
