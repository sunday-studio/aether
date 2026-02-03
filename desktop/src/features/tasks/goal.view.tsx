import { useQueryClient } from "@tanstack/react-query";
import { Loader } from "lucide-react";
import { Button } from "react-aria-components";
import { useParams } from "react-router";
import { useCreateTask, useGetGoalById, useGetGoalInstancesInfinite } from "~/aether-sdk";
import { useInfiniteScroll } from "~/hooks/use-infinite-scroll";
import { GoalFormDialog } from "./components/goals/goal-form-dialog";
import { RecurrencyTag } from "./components/goals/recurrency-tag";
import { VirtualizedTaskList } from "./components/virtualized-task-list";
import { invalidateTaskQueries } from "./invalidate-task-queries";
import { transformGoalInstancesToGroupedTasks } from "./tasks.domain";

export const GoalView = () => {
	const { goalId } = useParams();
	const queryClient = useQueryClient();
	const { data: goal, isLoading: isLoadingGoal } = useGetGoalById(goalId ?? "");

	const {
		data: goalInstancesData,
		isLoading: isLoadingGoalInstances,
		fetchNextPage,
		hasNextPage,
		isFetchingNextPage,
	} = useGetGoalInstancesInfinite(
		goalId ?? "",
		{},
		{
			query: {
				getNextPageParam: (lastPage) => lastPage.data?.nextCursor ?? undefined,
			},
		},
	);
	const { mutate: createTask } = useCreateTask();

	const isLoading = isLoadingGoal || isLoadingGoalInstances;

	const { items: allGoalInstances, hasMore, isFetchingMore, fetchMore } = useInfiniteScroll({
		pages: goalInstancesData?.pages,
		getItems: (page) => page.data?.items ?? [],
		fetchNextPage,
		hasNextPage,
		isFetchingNextPage,
	});

	const groupedGoalInstances = transformGoalInstancesToGroupedTasks(allGoalInstances);

	const handleCreateTask = () => {
		createTask(
			{
				data: {
					title: "New Task",
					goalId: goalId ?? "",
				},
			},
			{
				onSuccess: () =>
					invalidateTaskQueries(queryClient, {
						goalId: goalId ?? undefined,
					}),
			},
		);
	};

	if (isLoading) {
		return (
			<div className="h-full flex items-center justify-center">
				<Loader className="w-4 h-4 animate-spin" />
			</div>
		);
	}

	return (
		<div className="h-full">
			<div className="flex items-start justify-between py-4 gap-5">
				<div className="flex items-start flex-col gap-1">
					<h3 className="font-gt-ultra text-2xl font-medium">
						{goal?.data?.name}
					</h3>
					{goal?.data?.description && (
						<p className="text-sm text-neutral-500">
							{goal?.data?.description}
						</p>
					)}
					<RecurrencyTag recurrenceType={goal?.data?.recurrenceType ?? ""} />
				</div>

				<div className="flex items-center gap-1 shrink-0 ring ring-neutral-200 rounded-lg px-0.5 py-0.5 text-xs">
					<Button
						onClick={handleCreateTask}
						className="cursor-pointer p-1 px-1.5 hover:bg-neutral-200 rounded-md"
					>
						Add task
					</Button>
					<GoalFormDialog
						goal={goal?.data}
						trigger={
							<div className="cursor-pointer p-1 px-1.5 hover:bg-neutral-200 rounded-md">
								Edit
							</div>
						}
					/>
				</div>
			</div>
			<VirtualizedTaskList
				groupedTasks={groupedGoalInstances}
				infiniteScroll={{
					hasMore,
					isFetchingMore,
					fetchMore,
				}}
			/>
		</div>
	);
};
