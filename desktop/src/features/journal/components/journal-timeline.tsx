import { useGetEntries } from "~/aether-sdk";
import type { DbEntry } from "~/aether-sdk/models";
import { AddNewButton } from "~/components/shared/button";
import { Timeline } from "~/components/shared/timeline";
import { useCreateJournalEntry } from "~/hooks/use-create-journal-entry.ts";
import { sortEntries } from "../journal.domain.ts";
import { JournalTimelineItem } from "./journal-timeline-item.tsx";

export const JournalTimeline = () => {
	const { data: entries } = useGetEntries();
	const { createEntry } = useCreateJournalEntry();

	const sortedEntries = sortEntries(
		(entries?.data as unknown as DbEntry[]) ?? [],
	);

	return (
		<div className="h-full overflow-y-scroll bg-neutral-50 relative flex justify-center mt-2 mb-100! bg=">
			<Timeline>
				<Timeline.Item
					className="max-w-5xl bg-red-0 pt-6"
					indicatorContainerClassName="w-10"
					leftContainerClassName="w-40"
					rightContent={
						<Timeline.RightContent className="pb-10">
							<AddNewButton
								onClick={createEntry}
								label="Write"
								shortcuts={["⌘", "N"]}
							/>
						</Timeline.RightContent>
					}
				/>
				{sortedEntries?.map((entry) => {
					return <JournalTimelineItem key={entry.id} entry={entry} />;
				})}
			</Timeline>
		</div>
	);
};
