import { useQueryClient } from "@tanstack/react-query";
import {
	getGetEntriesQueryKey,
	useCreateEntry,
	useGetEntries,
} from "~/aether-sdk";
import type { DbEntry } from "~/aether-sdk/models";
import { Timeline } from "~/components/shared/timeline";
import { cn } from "~/utils/cn.ts";
import { sortEntries } from "../journal.domain.ts";
import { JournalTimelineItem } from "./journal-timeline-item.tsx";

const placeholder =
	'{"root":{"children":[{"children":[],"direction":"ltr","format":"","indent":0,"type":"paragraph","version":1,"textFormat":0,"textStyle":""}],"direction":"ltr","format":"","indent":0,"type":"root","version":1}}';

export const AddNewEntryButton = ({ onClick }: { onClick: () => void }) => {
	return (
		<button
			className={cn(
				"ring ring-neutral-200 text-neutral-700 flex items-center gap-1",
				"px-3 py-1.5 text-sm rounded-full bg-neutral-100",
				"hover:ring-neutral-300",
				"ring-3 transition-all duration-200 cursor-pointer",
			)}
			type="button"
			onClick={onClick}
		>
			Write
			<div className="flex items-center justify-center gap-0.5">
				<kbd className="px-1 h-5 w-fit min-w-5 rounded-md text-xs font-medium pointer-events-none  inline-flex items-center justify-center gap-1 bg-neutral-200 text-center select-none">
					⌘
				</kbd>
				<kbd className="px-1 h-5 w-fit min-w-5 rounded-md text-xs font-medium pointer-events-none  inline-flex items-center justify-center gap-1 bg-neutral-200 text-center select-none">
					N
				</kbd>
			</div>
		</button>
	);
};

export const JournalTimeline = () => {
	const queryClient = useQueryClient();

	const { data: entries } = useGetEntries();

	const sortedEntries = sortEntries(
		(entries?.data as unknown as DbEntry[]) ?? [],
	);

	const { mutate: createEntry } = useCreateEntry();
	const entriesQueryKey = getGetEntriesQueryKey();

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
				onSuccess: () => {
					queryClient.invalidateQueries({ queryKey: entriesQueryKey });
				},
				onError: (error) => {
					console.log("error ->", error);
					console.error(error);
				},
			},
		);
	};

	return (
		<div className="h-full overflow-y-scroll bg-neutral-50 relative flex justify-center ">
			<div className="my-10 ">
				<Timeline>
					<Timeline.Item
						className="grid-cols-24 grid"
						leftContainerClassName="col-start-5 col-end-9"
						rightContainerClassName="col-start-10 col-end-20"
						rightContent={
							<Timeline.RightContent className="-my-1 pb-10">
								<AddNewEntryButton onClick={onAddNewEntry} />
							</Timeline.RightContent>
						}
					/>
					{sortedEntries?.map((entry) => {
						return <JournalTimelineItem key={entry.id} entry={entry} />;
					})}
				</Timeline>
			</div>
		</div>
	);
};
