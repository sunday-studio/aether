import { useQueryClient } from "@tanstack/react-query";
import { Loader } from "lucide-react";
import { useMemo } from "react";
import {
	getGetInboxTasksQueryKey,
	useCreateTask,
	useGetInboxTasksInfinite,
} from "~/aether-sdk";
import { Button } from "~/components/shared/button";
import { VirtualizedTaskList } from "./components/virtualized-task-list";
import { groupTaskByCreatedAt } from "./tasks.domain";

export const InboxTasksView = () => {
	const queryClient = useQueryClient();
	const inboxTasksQueryKey = getGetInboxTasksQueryKey();
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

	const { mutate: createTask } = useCreateTask();

	// Flatten all pages into a single array of tasks
	const allTasks = useMemo(() => {
		return inboxTasksData?.pages?.flatMap((page) => page.data?.items ?? []) ?? [];
	}, [inboxTasksData]);

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
		createTask(
			{
				data: {
					title: "New Task",
				},
			},
			{
				onSuccess: () => {
					queryClient.invalidateQueries({ queryKey: inboxTasksQueryKey });
				},
			},
		);
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
