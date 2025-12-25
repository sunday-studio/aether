import { useVirtualizer } from "@tanstack/react-virtual";
import { isBefore, startOfDay } from "date-fns";
import { useMemo, useRef } from "react";
import type { DbTask } from "~/aether-sdk/models";
import { cn } from "~/utils/cn";
import { TaskItem } from "./task-item/task-item";
import { TaskListDivider } from "./task-list-divider";

type VirtualItem =
	| { type: "divider"; date: string; tasks: DbTask[]; isPast: boolean }
	| { type: "task"; task: DbTask; isPast: boolean; isLastInGroup: boolean };

interface VirtualizedTaskListProps {
	groupedTasks: Record<string, DbTask[]>;
	getDividerTitle?: (date: string, tasks: DbTask[]) => string | undefined;
	showPastDateEffects?: boolean;
	className?: string;
	emptyState?: React.ReactNode;
}

export const VirtualizedTaskList = ({
	groupedTasks,
	getDividerTitle,
	showPastDateEffects = true,
	className,
	emptyState,
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

		return items;
	}, [groupedTasks, showPastDateEffects]);

	const virtualizer = useVirtualizer({
		count: virtualItems.length,
		getScrollElement: () => parentRef.current,
		estimateSize: (index) => {
			const item = virtualItems[index];
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
							}}
							className={cn(
								"px-0.5 pb-4 transform-gpu transition-opacity duration-200",
								item.isPast && "opacity-60 hover:opacity-100",
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
