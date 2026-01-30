import { useGetOverdueTasksInfinite } from "~/aether-sdk";
import { useInfiniteScroll } from "~/hooks/use-infinite-scroll";
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

	const { items: allTasks, hasMore, isFetchingMore, fetchMore } = useInfiniteScroll({
		pages: overdueTasksData?.pages,
		getItems: (page) => page.data?.items ?? [],
		fetchNextPage,
		hasNextPage,
		isFetchingNextPage,
	});

	const groupedTasks = groupTaskByCreatedAt(allTasks);

	return (
		<div>
			<div className="flex items-center justify-between py-4">
				<h3 className="font-gt-ultra text-2xl font-medium">Overdue tasks</h3>
			</div>
			<VirtualizedTaskList
				groupedTasks={groupedTasks}
				infiniteScroll={{
					hasMore,
					isFetchingMore,
					fetchMore,
				}}
			/>
		</div>
	);
};
