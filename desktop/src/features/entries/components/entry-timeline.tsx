import { useEffect, useRef, useState } from "react";
import { EntryTimelineItem } from "./entry-timeline-item.tsx";

import { useGetEntry } from "~/aether-sdk";
import { normalizeEntries, generateDays, getDateKey } from "../entries.domain";

export const EntryTimeline = () => {
	const initialDays = generateDays(new Date(), 1);

	const [days, setDays] = useState(initialDays);

	const containerRef = useRef<HTMLDivElement>(null);
	const todayRef = useRef<HTMLDivElement>(null);

	const { data: entries } = useGetEntry();
	const normalizedEntries = normalizeEntries(entries?.data ?? []);

	// console.log("normalizedEntries ->", normalizedEntries);

	// Scroll to today on first mount
	useEffect(() => {
		if (todayRef.current) {
			todayRef.current.scrollIntoView({
				block: "start",
				behavior: "auto",
			});
		}
	}, []);

	return (
		<div
			ref={containerRef}
			className="h-full overflow-y-scroll bg-neutral-50 relative"
		>
			{days.map((date) => {
				const dateKey = getDateKey(date);
				const entries = normalizedEntries[dateKey];
				return <EntryTimelineItem key={dateKey} date={date} data={entries} />;
			})}
		</div>
	);
};
