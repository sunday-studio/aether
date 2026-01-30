import { Loader } from "lucide-react";
import { useMemo } from "react";
import { useGetOverdueTasksInfinite } from "~/aether-sdk";
import { VirtualizedTaskList } from "./components/virtualized-task-list";
import { groupTaskByCreatedAt } from "./tasks.domain";

export const OverdueTasksView = () => {
	const {
		data: overdueTasksData,
		fetchNextPage,
		hasNextPage,
		isFetchingNextPage,
	} = useGetOverdueTasksInfinite(
		{},
		{
			query: {
				getNextPageParam: (lastPage) => lastPage.data?.nextCursor ?? undefined,
			},
		},
	);

	// Flatten all pages into a single array of tasks
	const allTasks = useMemo(() => {
		return overdueTasksData?.pages?.flatMap((page) => page.data?.items ?? []) ?? [];
	}, [overdueTasksData]);

	const groupedTasks = groupTaskByCreatedAt(allTasks);

	return (
		<div>
			<div className="flex items-center justify-between py-4">
				<h3 className="font-gt-ultra text-2xl font-medium">Overdue tasks</h3>
			</div>
			<VirtualizedTaskList groupedTasks={groupedTasks} />
			{hasNextPage && (
				<div className="py-4 flex justify-center">
					<button
						type="button"
						onClick={() => fetchNextPage()}
						disabled={isFetchingNextPage}
						className="text-sm text-neutral-500 hover:text-neutral-700 disabled:opacity-50 flex items-center gap-2"
					>
						{isFetchingNextPage ? (
							<Loader className="w-4 h-4 animate-spin" />
						) : (
							"Load more"
						)}
					</button>
				</div>
			)}
		</div>
	);
};
