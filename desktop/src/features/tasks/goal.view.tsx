import { useQueryClient } from "@tanstack/react-query";
import { Loader, Plus } from "lucide-react";
import { Button } from "react-aria-components";
import { useParams } from "react-router";
import {
	getGetGoalInstancesQueryKey,
	useCreateTask,
	useGetGoalByID,
	useGetGoalInstances,
} from "~/aether-sdk";
import { RecurrencyTag } from "./components/goals/recurrency-tag";
import { VirtualizedTaskList } from "./components/virtualized-task-list";
import { transformGoalInstancesToGroupedTasks } from "./tasks.domain";

export const GoalView = () => {
	const { goalId } = useParams();
	const { data: goal, isLoading: isLoadingGoal } = useGetGoalByID(goalId ?? "");

	const queryClient = useQueryClient();
	const goalInstancesQueryKey = getGetGoalInstancesQueryKey(goalId ?? "");

	const { data: goalInstances, isLoading: isLoadingGoalInstances } =
		useGetGoalInstances(goalId ?? "");
	const { mutate: createTask } = useCreateTask();

	const isLoading = isLoadingGoal || isLoadingGoalInstances;

	const groupedGoalInstances = transformGoalInstancesToGroupedTasks(
		goalInstances?.data ?? [],
	);

	const handleCreateTask = () => {
		createTask(
			{
				data: {
					title: "New Task",
					goalId: goalId ?? "",
				},
			},
			{
				onSuccess: () => {
					queryClient.invalidateQueries({
						queryKey: goalInstancesQueryKey,
					});
				},
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
			<div className="flex items-start justify-between py-4 gap-1">
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

				<Button
					onClick={handleCreateTask}
					className="hover:bg-neutral-200 rounded-lg bg-neutral-100 cursor-pointer p-1"
				>
					<Plus size={16} strokeWidth={3} />
				</Button>
			</div>
			<VirtualizedTaskList groupedTasks={groupedGoalInstances} />
		</div>
	);
};
