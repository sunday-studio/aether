import {
	addMonths,
	eachDayOfInterval,
	endOfMonth,
	endOfWeek,
	format,
	isSameMonth,
	isToday,
	startOfMonth,
	startOfWeek,
	subMonths,
} from "date-fns";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useMemo, useState, useRef, useEffect } from "react";
import { useGetActivities } from "~/aether-sdk";
import { cn } from "~/utils/cn";
import { Tooltip } from "./tooltip";

type ActivityData = Record<string, Record<string, Record<string, number>>>;

export const ActivityHeatmap = () => {
	const [currentMonth, setCurrentMonth] = useState(new Date());
	const [isVisible, setIsVisible] = useState(false);
	const timeoutRef = useRef<NodeJS.Timeout | null>(null);
	const containerRef = useRef<HTMLDivElement>(null);

	const monthStart = startOfMonth(currentMonth);
	const monthEnd = endOfMonth(currentMonth);
	const calendarStart = startOfWeek(monthStart, { weekStartsOn: 0 });
	const calendarEnd = endOfWeek(monthEnd, { weekStartsOn: 0 });

	// Calculate date range for API call (start of calendar to end of calendar)
	const startDate = calendarStart.toISOString();
	const endDate = calendarEnd.toISOString();

	const { data: activitiesResponse } = useGetActivities({
		start_date: startDate,
		end_date: endDate,
	});

	console.log(activitiesResponse);

	const activities = activitiesResponse?.data as ActivityData | undefined;

	// Generate all days in the calendar view
	const days = useMemo(() => {
		return eachDayOfInterval({ start: calendarStart, end: calendarEnd });
	}, [calendarStart, calendarEnd]);

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

	const monthName = format(currentMonth, "MMMM yyyy");

	// Group days by week
	const weeks = useMemo(() => {
		const weeksArray: Date[][] = [];
		let currentWeek: Date[] = [];

		days.forEach((day, index) => {
			currentWeek.push(day);
			if ((index + 1) % 7 === 0) {
				weeksArray.push(currentWeek);
				currentWeek = [];
			}
		});

		return weeksArray;
	}, [days]);

	// Handle mouse enter with immediate show
	const handleMouseEnter = () => {
		if (timeoutRef.current) {
			clearTimeout(timeoutRef.current);
			timeoutRef.current = null;
		}
		setIsVisible(true);
	};

	// Handle mouse leave with delayed hide to prevent flickering
	const handleMouseLeave = () => {
		timeoutRef.current = setTimeout(() => {
			setIsVisible(false);
		}, 100); // Small delay to prevent flickering when moving between elements
	};

	// Cleanup timeout on unmount
	useEffect(() => {
		return () => {
			if (timeoutRef.current) {
				clearTimeout(timeoutRef.current);
			}
		};
	}, []);

	return (
		<div
			ref={containerRef}
			onMouseEnter={handleMouseEnter}
			onMouseLeave={handleMouseLeave}
			className={cn(
				"w-auto p-4 rounded-lg border border-neutral-200 absolute z-10 transition-all duration-300 bg-white",
				isVisible ? "bottom-5 left-5" : "-bottom-20 -left-20"
			)}
		>
			{/* <div className="flex items-center justify-between mb-4">
				<button
					type="button"
					onClick={handlePreviousMonth}
					className="p-1.5 rounded-full hover:bg-neutral-100 transition-colors"
					aria-label="Previous month"
				>
					<ChevronLeft className="w-5 h-5 text-neutral-600" />
				</button>

				<h3 className="text-sm font-medium text-neutral-700">
					{monthName}
				</h3>

				<button
					type="button"
					onClick={handleNextMonth}
					className="p-1.5 rounded-full hover:bg-neutral-100 transition-colors"
					aria-label="Next month"
				>
					<ChevronRight className="w-5 h-5 text-neutral-600" />
				</button>
			</div> */}

			{/* Week day labels */}
			{/* <div className="grid grid-cols-7 gap-1 mb-2">
				{["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"].map((day) => (
					<div
						key={day}
						className="text-xs text-center text-neutral-500 font-medium"
					>
						{day}
					</div>
				))}
			</div> */}

			{/* Calendar grid */}
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
											"col-span-1 w-3 h-3 aspect-square rounded-sm transition-colors",
											getColorClass(intensity),
											!isCurrentMonth && "opacity-30",
											// isCurrentDay && "ring-2 ring-green-500",
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
	);
};
