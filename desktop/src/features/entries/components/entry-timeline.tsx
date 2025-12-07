import { useEffect, useRef, useState } from "react";
import { useGetEntry } from "~/aether-sdk";
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

	const { data: entries } = useGetEntry();
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
			<div className="my-10 w-[650px]">
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
						return (
							<Timeline.Item key={entry.id}>
								<Timeline.Indicator className="cursor-pointer" />
								<Timeline.Content className="mb-5">
									<EntryEditor
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
					})}
				</Timeline>
			</div>
		</div>
	);
};
