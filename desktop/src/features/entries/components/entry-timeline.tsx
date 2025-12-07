import { useEffect, useRef, useState } from "react";
import { useGetEntries } from "~/aether-sdk";
import type { DbEntry } from "~/aether-sdk/models";
import { Timeline } from "~/components/shared/timeline";
import { generateDays, sortEntries } from "../entries.domain";
import { EntryEditor } from "./entry-editor.tsx";
import {
	AddNewEntryButton,
	EntryTimelineItem,
} from "./entry-timeline-item.tsx";

export const EntryTimeline = () => {
	const initialDays = generateDays(new Date(), 1);

	const [_days, _setDayss] = useState(initialDays);

	const containerRef = useRef<HTMLDivElement>(null);
	const todayRef = useRef<HTMLDivElement>(null);

	const { data: entries } = useGetEntries();
	const sortedEntries = sortEntries(
		(entries?.data as unknown as DbEntry[]) ?? [],
	);

	// useEffect(() => {
	// 	if (todayRef.current) {
	// 		todayRef.current.scrollIntoView({
	// 			block: "start",
	// 			behavior: "auto",
	// 		});
	// 	}
	// }, []);

	return (
		<div
			ref={containerRef}
			className="h-full overflow-y-scroll bg-neutral-50 relative flex justify-center "
		>
			<div className="my-10 w-[700px]">
				<Timeline>
					<Timeline.Item>
						<Timeline.Indicator />
						<Timeline.Content className="-my-1">
							<AddNewEntryButton
								onClick={() => {
									console.log("add new entry");
								}}
							/>
						</Timeline.Content>
					</Timeline.Item>
					{sortedEntries?.map((entry) => {
						return <EntryTimelineItem key={entry.id} entry={entry} />;
					})}
				</Timeline>
			</div>
		</div>
	);
};
