import { Loader } from "lucide-react";
import { useGetInboxTasksInfinite } from "~/aether-sdk";
import { Button } from "~/components/shared/button";
import { useInfiniteScroll } from "~/hooks/use-infinite-scroll";
import { VirtualizedTaskList } from "./components/virtualized-task-list";
import { groupTaskByCreatedAt } from "./tasks.domain";
import { useOptimisticCreateTask } from "./use-optimistic-task-hooks";

export const InboxTasksView = () => {
	const {
		data: inboxTasksData,
		isLoading: isLoadingInboxTasks,
		error: errorInboxTasks,
		fetchNextPage,
		hasNextPage,
		isFetchingNextPage,
	} = useGetInboxTasksInfinite(
		{},
		{
			query: {
				getNextPageParam: (lastPage) => lastPage.data?.nextCursor ?? undefined,
			},
		},
	);

	const { mutate: createTask } = useOptimisticCreateTask();

	// Use the infinite scroll hook to flatten pages and get scroll helpers
	const { items: allTasks, hasMore, isFetchingMore, fetchMore } = useInfiniteScroll({
		pages: inboxTasksData?.pages,
		getItems: (page) => page.data?.items ?? [],
		fetchNextPage,
		hasNextPage,
		isFetchingNextPage,
	});

	if (errorInboxTasks) {
		return (
			<div className="h-full flex items-center justify-center">
				<p className="text-sm text-neutral-500">Error loading inbox tasks</p>
			</div>
		);
	}

	if (isLoadingInboxTasks) {
		return (
			<div className="h-full flex items-center justify-center">
				<Loader className="w-4 h-4 animate-spin" />
			</div>
		);
	}

	const groupedTasks = groupTaskByCreatedAt(allTasks);

	const handleCreateTask = () => {
		createTask({
			data: {
				title: "New Task",
			},
		});
	};

	return (
		<div className="h-full flex flex-col">
			<div className="flex items-center justify-between py-4">
				<h3 className="font-gt-ultra text-2xl font-medium">Inbox</h3>
				<Button
					onClick={handleCreateTask}
					label="Add task"
					tooltipContent="Add a new task"
					shortcuts={["⌘", "N"]}
				/>
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
