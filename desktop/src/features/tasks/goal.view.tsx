import { useParams } from "react-router";
import { useGetGoalByID, useGetGoalInstances } from "~/aether-sdk";
import { AddNewButton } from "~/components/shared/button";
import { TasksContainer } from "./components/task-item/tasks-container";

export const GoalView = () => {
	const { goalId } = useParams();
	const { data: goal } = useGetGoalByID(goalId ?? "");
	const { data: goalInstances } = useGetGoalInstances(goalId ?? "");

	console.log("goalInstances", goalInstances);

	return (
		<div className="h-full">
			<div className="flex items-start flex-col justify-between py-4 gap-1">
				<h3 className="newsreader-font text-2xl font-medium">
					{goal?.data?.name}
				</h3>
				<p className="text-sm text-neutral-500">{goal?.data?.description}</p>
				{/* <AddNewButton
					onClick={handleCreateTask}
					label="Add task"
					shortcuts={["⌘", "N"]}
				/> */}
			</div>
			<ul className="w-full h-full overflow-y-scroll">
				{/* {Object.entries(groupedTasks).map(([date, tasks]) => {
					return <TasksContainer key={date} date={date} tasks={tasks} />;
				})} */}
			</ul>
		</div>
	);
};
