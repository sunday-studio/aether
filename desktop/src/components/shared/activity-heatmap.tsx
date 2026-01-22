import {
	addDays,
	addMonths,
	format,
	isSameMonth,
	isToday,
	startOfMonth,
	startOfWeek,
	subMonths,
} from "date-fns";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useCallback, useMemo, useState } from "react";
import { useGetActivities } from "~/aether-sdk";
import { cn } from "~/utils/cn";
import { Tooltip } from "./tooltip";

type ActivityData = Record<string, Record<string, Record<string, number>>>;

const WEEKS_TO_SHOW = 6;
const DAYS_PER_WEEK = 7;
const TOTAL_DAYS = WEEKS_TO_SHOW * DAYS_PER_WEEK;

export const ActivityHeatmap = () => {
	const [currentMonth, setCurrentMonth] = useState(new Date());

	const monthStart = startOfMonth(currentMonth);
	const calendarStart = startOfWeek(monthStart, { weekStartsOn: 0 });
	const calendarEnd = addDays(calendarStart, TOTAL_DAYS - 1);

	const { data: activitiesResponse } = useGetActivities({
		start_date: calendarStart.toISOString(),
		end_date: calendarEnd.toISOString(),
	});

	const activities = activitiesResponse?.data as ActivityData | undefined;

	const getActivityCount = useCallback(
		(date: Date): number => {
			if (!activities) return 0;

			const dateKey = format(date, "yyyy-MM-dd");
			const dayActivities = activities[dateKey];
			if (!dayActivities) return 0;

			let total = 0;
			for (const entityType in dayActivities) {
				for (const actionType in dayActivities[entityType]) {
					total += dayActivities[entityType][actionType] || 0;
				}
			}
			return total;
		},
		[activities],
	);

	const maxActivityCount = useMemo(() => {
		if (!activities) return 0;

		let max = 0;
		for (let dayIndex = 0; dayIndex < TOTAL_DAYS; dayIndex++) {
			const day = addDays(calendarStart, dayIndex);
			const count = getActivityCount(day);
			if (count > max) {
				max = count;
			}
		}
		return max;
	}, [activities, calendarStart, getActivityCount]);

	const getIntensity = (count: number): number => {
		if (count === 0 || maxActivityCount === 0) return 0;

		const ratio = count / maxActivityCount;
		if (ratio <= 0.25) return 1;
		if (ratio <= 0.5) return 2;
		if (ratio <= 0.75) return 3;
		return 4;
	};

	const getColorClass = (intensity: number): string => {
		const colors = [
			"bg-neutral-100",
			"bg-green-100",
			"bg-green-300",
			"bg-green-500",
			"bg-green-700",
		];
		return colors[intensity] || colors[0];
	};

	const monthName = format(currentMonth, "MMM yy");

	const weeks = useMemo(() => {
		return Array.from({ length: WEEKS_TO_SHOW }, (_, weekIndex) => {
			const weekStart = addDays(calendarStart, weekIndex * DAYS_PER_WEEK);
			return Array.from({ length: DAYS_PER_WEEK }, (_, dayIndex) =>
				addDays(weekStart, dayIndex),
			);
		});
	}, [calendarStart]);

	return (
		<div className="absolute z-10 -bottom-30 -left-30 p-5 duration-300 ease-out hover:bottom-0 hover:left-0 group">
			<div className="flex items-center justify-between mb-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200 ease-out">
				<h3 className="text-xs text-(--color-foreground)">{monthName}</h3>
				<div className="flex gap-1">
					<button
						type="button"
						onClick={() => setCurrentMonth(subMonths(currentMonth, 1))}
						className="p-1.5 rounded-full hover:bg-neutral-200 transition-colors"
						aria-label="Previous month"
					>
						<ChevronLeft
							className="w-3 h-3 text-(--color-foreground)"
							strokeWidth={3}
						/>
					</button>
					<button
						type="button"
						onClick={() => setCurrentMonth(addMonths(currentMonth, 1))}
						className="p-1.5 rounded-full hover:bg-neutral-200 transition-colors"
						aria-label="Next month"
					>
						<ChevronRight
							className="w-3 h-3 text-(--color-foreground)"
							strokeWidth={3}
						/>
					</button>
				</div>
			</div>

			<div className="w-auto p-3 navigation-control rounded-lg bg-(--color-card)">
				<div className="grid grid-cols-7 gap-1">
					{weeks.map((week) =>
						week.map((day) => {
							const count = getActivityCount(day);
							const intensity = getIntensity(count);
							const isCurrentMonth = isSameMonth(day, currentMonth);
							const isCurrentDay = isToday(day);
							const dayKey = format(day, "yyyy-MM-dd");

							return (
								<Tooltip
									key={dayKey}
									content={
										count > 0
											? `${count} ${
													count === 1 ? "activity" : "activities"
												} on ${format(day, "MMM d, yyyy")}`
											: `No activities on ${format(day, "MMM d, yyyy")}`
									}
									trigger={
										<div
											className={cn(
												"col-span-1 w-3.5 h-3.5 aspect-square rounded-sm transition-colors",
												getColorClass(intensity),
												!isCurrentMonth && "opacity-30",
												isCurrentDay && "border-2 border-(--color-ring)",
											)}
										/>
									}
								/>
							);
						}),
					)}
				</div>
			</div>
		</div>
	);
};
