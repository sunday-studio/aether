import { Loader } from "lucide-react";
import { useParams } from "react-router";
import { useGetGoalByID, useGetGoalInstances } from "~/aether-sdk";
import { RecurrencyTag } from "./components/goals/recurrency-tag";
import { VirtualizedTaskList } from "./components/virtualized-task-list";
import { transformGoalInstancesToGroupedTasks } from "./tasks.domain";

export const GoalView = () => {
	const { goalId } = useParams();
	const { data: goal, isLoading: isLoadingGoal } = useGetGoalByID(goalId ?? "");
	const { data: goalInstances, isLoading: isLoadingGoalInstances } =
		useGetGoalInstances(goalId ?? "");

	const isLoading = isLoadingGoal || isLoadingGoalInstances;

	const groupedGoalInstances = transformGoalInstancesToGroupedTasks(
		goalInstances?.data ?? [],
	);

	if (isLoading) {
		return (
			<div className="h-full flex items-center justify-center">
				<Loader className="w-4 h-4 animate-spin" />
			</div>
		);
	}

	return (
		<div className="h-full">
			<div className="flex items-start flex-col justify-between py-4 gap-1">
				<h3 className="newsreader-font text-2xl font-medium">
					{goal?.data?.name}
				</h3>
				{goal?.data?.description && (
					<p className="text-sm text-neutral-500">{goal?.data?.description}</p>
				)}
				<RecurrencyTag recurrenceType={goal?.data?.recurrenceType ?? ""} />
			</div>
			<VirtualizedTaskList groupedTasks={groupedGoalInstances} />
		</div>
	);
};
