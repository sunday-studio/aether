import { Loader } from "lucide-react";
import { Button } from "~/components/shared/button";
import { Timeline } from "~/components/shared/timeline";
import { useCreateJournalEntry } from "~/hooks/use-create-journal-entry.ts";
import { useEntriesInfinite } from "~/hooks/use-entries-infinite";
import { sortEntries } from "../journal.domain.ts";
import { JournalTimelineItem } from "./journal-timeline-item.tsx";
import { JournalWeeklyAiSummary } from "./journal-weekly-ai-summary.tsx";

// TODO: show different message if user has no entries but has deleted entries

export const JournalTimeline = () => {
	const { items, sentinelRef, isFetchingMore } = useEntriesInfinite();
	const { createEntry } = useCreateJournalEntry();

	const sortedEntries = sortEntries(items);

	const hasNoEntries = sortedEntries.length === 0;

	if (hasNoEntries) {
		return (
			<div className="h-full relative flex justify-center items-center">
				<div className="flex items-center flex-col text-sm text-neutral-500">
					<p className="">
						First day? Welcome to{" "}
						<span className="text-neutral-800">Aether</span>.
					</p>
					<p className="mb-6">Let's start with a new journal entry.</p>
					<Button
						onClick={createEntry}
						label="Let's start"
						shortcuts={["⌘", "N"]}
						tooltipContent="Create the first one"
					/>
				</div>
			</div>
		);
	}

	return (
		<div className="h-full overflow-y-scroll  relative flex justify-center mt-2 mb-200!">
			<Timeline>
				<Timeline.Item
					className="max-w-5xl w-full bg-red-0 pt-6"
					indicatorContainerClassName="w-10"
					leftContainerClassName="w-50"
					rightContent={
						<Timeline.RightContent className="pb-10">
							<div className="flex items-center gap-2">
								<Button
									onClick={createEntry}
									label="Write"
									shortcuts={["⌘", "N"]}
									tooltipContent="Create a new entry"
								/>
								<JournalWeeklyAiSummary />
							</div>
						</Timeline.RightContent>
					}
				/>
				{sortedEntries?.map((entry) => {
					return <JournalTimelineItem key={entry.id} entry={entry} />;
				})}
			</Timeline>
			<div ref={sentinelRef} className="py-8 flex justify-center">
				{isFetchingMore && (
					<Loader className="w-4 h-4 animate-spin text-neutral-400" />
				)}
			</div>
		</div>
	);
};
