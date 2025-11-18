import { useEffect, useRef, useState } from "react";
import { format, addDays, isToday } from "date-fns";
import { triggerHapticFeedback } from "~/utils/haptic";

import { TimelineEntry } from "./timeline-entry";

export const Timeline = () => {
	const today = new Date();
	const [days, setDays] = useState([today]);

	const containerRef = useRef<HTMLDivElement>(null);
	const todayRef = useRef<HTMLDivElement>(null);
	const isLoadingRef = useRef(false);

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
			className="h-full overflow-y-scroll bg-gray-50 px-4 py-8"
		>
			{days.map((date) => (
				<TimelineEntry
					key={date.toISOString()}
					date={date}
					todayRef={todayRef}
				/>
				// <div
				// 	key={date.toISOString()}
				// 	ref={
				// 		format(date, "yyyy-MM-dd") === format(today, "yyyy-MM-dd")
				// 			? todayRef
				// 			: null
				// 	}
				// 	className="mb-8"
				// >
				// 	<div className="sticky top-0 bg-gray-50 py-2 z-10 font-semibold newreader-font">
				// 		{format(date, "EEEE, MMMM d")}
				// 	</div>

				// 	<div className="space-y-2 mt-4">
				// 		<div className="p-4 bg-white text-sm">
				// 			Example entry for {format(date, "MMMM d")}
				// 		</div>
				// 	</div>
				// </div>
			))}
		</div>
	);
};
