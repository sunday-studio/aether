import { useVirtualizer } from "@tanstack/react-virtual";
import { isBefore, startOfDay } from "date-fns";
import { Loader } from "lucide-react";
import { useEffect, useMemo, useRef } from "react";
import type { TaskWithSubtasks } from "~/aether-sdk/models";
import { cn } from "~/utils/cn";
import { TaskItem } from "./task-item/task-item";
import { TaskListDivider } from "./task-list-divider";

type VirtualItem =
	| {
			type: "divider";
			date: string;
			tasks: TaskWithSubtasks[];
			isPast: boolean;
	  }
	| {
			type: "task";
			task: TaskWithSubtasks;
			isPast: boolean;
			isLastInGroup: boolean;
	  }
	| { type: "loader" };

interface InfiniteScrollProps {
	/** Whether there are more items to fetch */
	hasMore: boolean;
	/** Whether we're currently fetching more items */
	isFetchingMore: boolean;
	/** Function to fetch more items */
	fetchMore: () => void;
	/** Number of items from the end to trigger fetch (default: 5) */
	threshold?: number;
}

interface VirtualizedTaskListProps {
	groupedTasks: Record<string, TaskWithSubtasks[]>;
	getDividerTitle?: (
		date: string,
		tasks: TaskWithSubtasks[],
	) => string | undefined;
	showPastDateEffects?: boolean;
	className?: string;
	emptyState?: React.ReactNode;
	/** Optional infinite scroll configuration */
	infiniteScroll?: InfiniteScrollProps;
}

export const VirtualizedTaskList = ({
	groupedTasks,
	getDividerTitle,
	showPastDateEffects = true,
	className,
	emptyState,
	infiniteScroll,
}: VirtualizedTaskListProps) => {
	const parentRef = useRef<HTMLDivElement>(null);

	// Flatten the grouped tasks into a single array for virtualization
	const virtualItems = useMemo(() => {
		const items: VirtualItem[] = [];

		Object.entries(groupedTasks).forEach(([date, tasks]) => {
			const isPast =
				showPastDateEffects &&
				isBefore(startOfDay(new Date(date)), startOfDay(new Date()));

			// Add divider
			items.push({ type: "divider", date, tasks, isPast });

			// Add tasks
			tasks.forEach((task, index) => {
				items.push({
					type: "task",
					task,
					isPast,
					isLastInGroup: index === tasks.length - 1,
				});
			});
		});

		// Add loader item at the end if infinite scroll is enabled and has more
		if (infiniteScroll?.hasMore) {
			items.push({ type: "loader" });
		}

		return items;
	}, [groupedTasks, showPastDateEffects, infiniteScroll?.hasMore]);

	const virtualizer = useVirtualizer({
		count: virtualItems.length,
		getScrollElement: () => parentRef.current,
		estimateSize: (index) => {
			const item = virtualItems[index];
			if (item.type === "loader") {
				return 48; // Loader height
			}
			// Divider has my-6 (24px top + 24px bottom) + content (~28px) = ~76px
			// Task items are roughly 60px with space-y-4 (16px) gap
			if (item.type === "divider") {
				return 76;
			}
			// Task height + gap (16px for space-y-4)
			return 60 + 16;
		},
		overscan: 5,
	});

	// Trigger fetch when scrolling near the end
	useEffect(() => {
		if (!infiniteScroll) return;

		const {
			hasMore,
			isFetchingMore,
			fetchMore,
			threshold = 5,
		} = infiniteScroll;
		const virtualItems_visible = virtualizer.getVirtualItems();
		const lastItem = virtualItems_visible[virtualItems_visible.length - 1];

		if (!lastItem) return;

		// Check if we're near the end (within threshold items)
		const isNearEnd = lastItem.index >= virtualItems.length - threshold - 1;

		if (isNearEnd && hasMore && !isFetchingMore) {
			fetchMore();
		}
	}, [virtualizer.getVirtualItems(), virtualItems.length, infiniteScroll]);

	const isEmpty = Object.keys(groupedTasks).length === 0;

	if (isEmpty && emptyState) {
		return <>{emptyState}</>;
	}

	return (
		<div ref={parentRef} className={cn("flex-1 overflow-y-auto", className)}>
			<ul
				style={{
					height: `${virtualizer.getTotalSize()}px`,
					width: "100%",
					position: "relative",
				}}
			>
				{virtualizer.getVirtualItems().map((virtualRow) => {
					const item = virtualItems[virtualRow.index];

					if (item.type === "loader") {
						return (
							<li
								key="loader"
								data-index={virtualRow.index}
								ref={virtualizer.measureElement}
								style={{
									position: "absolute",
									top: 0,
									left: 0,
									width: "100%",
									transform: `translateY(${virtualRow.start}px)`,
								}}
								className="flex justify-center py-4"
							>
								{infiniteScroll?.isFetchingMore && (
									<Loader className="w-4 h-4 animate-spin text-neutral-400" />
								)}
							</li>
						);
					}

					if (item.type === "divider") {
						const dividerTitle = getDividerTitle?.(item.date, item.tasks);

						return (
							<li
								key={virtualRow.key}
								data-index={virtualRow.index}
								ref={virtualizer.measureElement}
								style={{
									position: "absolute",
									top: 0,
									left: 0,
									width: "100%",
									transform: `translateY(${virtualRow.start}px)`,
								}}
								className={cn(
									"px-0.5 transform-gpu transition-opacity duration-200",
									item.isPast && "opacity-60",
								)}
							>
								<TaskListDivider
									date={item.date}
									title={dividerTitle}
									completedCountLabel={`${item.tasks.filter((t) => t.isCompleted).length} / ${item.tasks.length}`}
								/>
							</li>
						);
					}

					return (
						<li
							key={virtualRow.key}
							data-index={virtualRow.index}
							ref={virtualizer.measureElement}
							style={{
								position: "absolute",
								top: 0,
								left: 0,
								width: "100%",
								transform: `translateY(${virtualRow.start}px)`,
								willChange: item.isPast ? "opacity" : undefined,
							}}
							className={cn(
								"px-0.5 pb-4 transform-gpu transition-opacity duration-200",
								item.isPast &&
									"opacity-60 hover:opacity-100 focus-within:opacity-100",
							)}
						>
							<TaskItem task={item.task} />
						</li>
					);
				})}
			</ul>
		</div>
	);
};
