import { format, isToday as isTodayFn } from "date-fns";
import clsx from "clsx";
import { EntryEditor } from "./entry-editor";
import type { DbEntry } from "~/aether-sdk/models";
import { useCreateEntry } from "~/aether-sdk";
import { Timeline } from "~/components/shared/timeline";
import { useState } from "react";

interface EntryTimelineItemProps {
	date: Date;
	data?: DbEntry[] | [];
}

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
	// console.log("entries ->", data);
	// const normalizedEntries = normalizeEntriesToMap(data);
	// const [entries, setEntries] = useState<Map<string, DbEntry[]>>();

	const [entries, setEntries] = useState<DbEntry[]>(data);
	const isToday = isTodayFn(date);
	const hasEntries = entries.length > 0;
	const [isAddingNewEntry, setIsAddingNewEntry] = useState(false);
	const { mutate: createEntry } = useCreateEntry();

	const onAddNewEntry = async () => {
		setIsAddingNewEntry(true);
		const now = new Date();

		createEntry(
			{
				data: {
					document: "",
					date: now.toISOString(),
				},
			},
			{
				onSuccess: (data) => {
					console.log("data ->", data);
					setEntries([...entries, data.data]);
				},
				onError: (error) => {
					console.log("error ->", error);
					console.error(error);
				},
			},
		);

		// setEntries([
		// 	...entries,
		// 	{
		// 		id: now.toISOString(),
		// 		createdAt: now.toISOString(),
		// 		updatedAt: now.toISOString(),
		// 		document: "",
		// 		isPinned: false,
		// 		isArchived: false,
		// 		isDeleted: false,
		// 	},
		// ]);
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
									<EntryEditor data={entry} />
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
