import { useParams } from "react-router";
import { useGetGoalByID, useGetGoalInstances } from "~/aether-sdk";
import { RecurrencyTag } from "./components/goals/recurrency-tag";
import { TasksContainer } from "./components/task-item/tasks-container";

export const GoalView = () => {
	const { goalId } = useParams();
	const { data: goal } = useGetGoalByID(goalId ?? "");
	const { data: goalInstances } = useGetGoalInstances(goalId ?? "");

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
			<ul className="w-full h-full overflow-y-scroll">
				{goalInstances?.data?.map((instance) => {
					const date = instance.periodStart
						? new Date(instance.periodStart).toISOString()
						: "";

					const isFirstInstance = instance.id === goalInstances?.data?.[0]?.id;
					return (
						<TasksContainer
							key={instance.id}
							// dividerTitle={title}
							date={date}
							tasks={instance.tasks ?? []}
							isPast={!isFirstInstance}
						/>
					);
				})}
			</ul>
		</div>
	);
};
