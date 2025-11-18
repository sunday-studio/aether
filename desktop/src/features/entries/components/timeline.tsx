import { RefObject, useEffect, useRef, useState } from "react";
import { format, addDays, isToday } from "date-fns";
import { triggerHapticFeedback } from "~/utils/haptic";
import { AnimatePresence } from "motion/react";
import { TimelineEntry } from "./timeline-entry";

import { useGetEntry } from "~/aether-sdk";
import { normalizeEntries } from "../entries.domain";

export const Timeline = () => {
	const today = new Date();
	const [days, setDays] = useState([today]);

	const containerRef = useRef<HTMLDivElement>(null);
	const todayRef = useRef<HTMLDivElement>(null);
	const isLoadingRef = useRef(false);

	const { data: entries } = useGetEntry();

	const normalizedEntries = normalizeEntries(entries?.data ?? []);

	console.log(normalizedEntries);

	// Scroll to today on first mount
	useEffect(() => {
		if (todayRef.current) {
			todayRef.current.scrollIntoView({
				block: "start",
				behavior: "auto",
			});
		}
	}, []);

	const onScroll = () => {
		const el = containerRef.current;
		if (!el || isLoadingRef.current) return;

		const top = el.scrollTop;
		const bottom = el.scrollHeight - el.clientHeight;

		if (top < 120) {
			isLoadingRef.current = true;

			const prevScrollHeight = el.scrollHeight;
			const prevScrollTop = el.scrollTop;

			const firstDay = days[0];
			const newDay = addDays(firstDay, -1);

			setDays((prev) => [newDay, ...prev]);

			// Wait for DOM to update
			requestAnimationFrame(() => {
				const newScrollHeight = el.scrollHeight;
				const heightDiff = newScrollHeight - prevScrollHeight;

				el.scrollTo({
					top: prevScrollTop + heightDiff,
					behavior: "smooth",
				});

				setTimeout(() => {
					isLoadingRef.current = false;
				}, 150);
			});

			return;
		}

		// ==== LOAD NEXT DAY (DOWN) ====
		if (top > bottom - 120) {
			isLoadingRef.current = true;

			const lastDay = days[days.length - 1];
			const newDay = addDays(lastDay, 1);

			setDays((prev) => [...prev, newDay]);

			requestAnimationFrame(() => {
				el.scrollTo({
					top: el.scrollTop + 60, // small nudge for smoothness
					behavior: "smooth",
				});

				setTimeout(() => {
					isLoadingRef.current = false;
				}, 150);
			});
		}
	};

	return (
		<div
			ref={containerRef}
			onScroll={onScroll}
			className="h-full overflow-y-scroll bg-gray-50 py-8 debug"
		>
			{days.map((date) => (
				<TimelineEntry
					key={date.toISOString()}
					date={date}
					todayRef={todayRef as RefObject<HTMLDivElement>}
					data={normalizedEntries[date.toISOString().split("T")[0]]}
				/>
			))}
		</div>
	);
};
