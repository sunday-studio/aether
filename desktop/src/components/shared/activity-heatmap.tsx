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
import { motion } from "motion/react";
import { useEffect, useMemo, useRef, useState } from "react";
import { useGetActivities } from "~/aether-sdk";
import { cn } from "~/utils/cn";
import { Tooltip } from "./tooltip";

type ActivityData = Record<string, Record<string, Record<string, number>>>;

export const ActivityHeatmap = () => {
	const [currentMonth, setCurrentMonth] = useState(new Date());
	const [isVisible, setIsVisible] = useState(false);

	const monthStart = startOfMonth(currentMonth);
	const calendarStart = startOfWeek(monthStart, { weekStartsOn: 0 });
	// Always show exactly 6 weeks (42 days = 6 weeks × 7 days)
	const calendarEnd = addDays(calendarStart, 41);

	// Calculate date range for API call (start of calendar to end of calendar)
	const startDate = calendarStart.toISOString();
	const endDate = calendarEnd.toISOString();

	const { data: activitiesResponse } = useGetActivities({
		start_date: startDate,
		end_date: endDate,
	});

	console.log(activitiesResponse);

	const activities = activitiesResponse?.data as ActivityData | undefined;

	// Calculate activity count for each day
	const getActivityCount = (date: Date): number => {
		if (!activities) return 0;

		const dateKey = format(date, "yyyy-MM-dd");
		const dayActivities = activities[dateKey];

		if (!dayActivities) return 0;

		// Sum all counts across all entity types and action types
		let total = 0;
		for (const entityType in dayActivities) {
			for (const actionType in dayActivities[entityType]) {
				total += dayActivities[entityType][actionType] || 0;
			}
		}

		return total;
	};

	// Get intensity level for color coding (0-4)
	const getIntensity = (count: number): number => {
		if (count === 0) return 0;
		if (count <= 2) return 1;
		if (count <= 5) return 2;
		if (count <= 10) return 3;
		return 4;
	};

	// Get color class based on intensity, remove dark classes
	const getColorClass = (intensity: number): string => {
		switch (intensity) {
			case 0:
				return "bg-neutral-100";
			case 1:
				return "bg-green-100";
			case 2:
				return "bg-green-300";
			case 3:
				return "bg-green-500";
			case 4:
				return "bg-green-700";
			default:
				return "bg-neutral-100";
		}
	};

	const handlePreviousMonth = () => {
		setCurrentMonth(subMonths(currentMonth, 1));
	};

	const handleNextMonth = () => {
		setCurrentMonth(addMonths(currentMonth, 1));
	};

	const monthName = format(currentMonth, "MMM yy");

	// Group days by week - always exactly 6 weeks
	const weeks = useMemo(() => {
		const weeksArray: Date[][] = [];

		// Always create exactly 6 weeks
		for (let weekIndex = 0; weekIndex < 6; weekIndex++) {
			const weekStart = addDays(calendarStart, weekIndex * 7);
			const week: Date[] = [];
			for (let dayIndex = 0; dayIndex < 7; dayIndex++) {
				week.push(addDays(weekStart, dayIndex));
			}
			weeksArray.push(week);
		}

		return weeksArray;
	}, [calendarStart]);

	return (
		<div className="absolute z-10  -bottom-30 -left-30 p-5 animate-in fade-in duration-200 ease-out hover:bottom-0 hover:left-0 group">
			<div className="flex items-center justify-between mb-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200 ease-out">
				<h3 className="text-xs text-neutral-700">{monthName}</h3>
				<div>
					<button
						type="button"
						onClick={handlePreviousMonth}
						className="p-1.5 rounded-full hover:bg-neutral-200 transition-colors"
						aria-label="Previous month"
					>
						<ChevronLeft
							className="w-3	 h-3 text-neutral-600"
							strokeWidth={3}
						/>
					</button>

					<button
						type="button"
						onClick={handleNextMonth}
						aria-label="Next month"
						className="p-1.5 rounded-full hover:bg-neutral-200 transition-colors"
					>
						<ChevronRight
							className="w-3 h-3 text-neutral-600"
							strokeWidth={3}
						/>
					</button>
				</div>
			</div>

			<div className="w-auto p-3 navigation-control rounded-lg bg-white">
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
												isCurrentDay && "border-2 border-green-500",
											)}
										/>
									}
								/>
							);
						}),
					)}
				</div>

				{/* Legend */}
				{/* <div className="flex items-center justify-end gap-1 mt-4">
				<span className="text-xs text-neutral-500 mr-2">
					Less
				</span>
				<div className="flex gap-1">
					{[0, 1, 2, 3, 4].map((intensity) => (
						<div
							key={intensity}
							className={cn("w-3 h-3 rounded-sm", getColorClass(intensity))}
						/>
					))}
				</div>
				<span className="text-xs text-neutral-500 ml-2">
					More
				</span>
			</div> */}
			</div>
		</div>
	);
};
