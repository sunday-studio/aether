import { useGetOverdueTasks } from "~/aether-sdk";
import { VirtualizedTaskList } from "./components/virtualized-task-list";
import { groupTaskByCreatedAt } from "./tasks.domain";

export const OverdueTasksView = () => {
	const { data: overdueTasks } = useGetOverdueTasks();

	const groupedTasks = groupTaskByCreatedAt(overdueTasks?.data ?? []);

	return (
		<div className="px-0.5">
			<div className="flex items-center justify-between py-4">
				<h3 className="newsreader-font text-2xl font-medium">Overdue Tasks</h3>
			</div>
			<VirtualizedTaskList groupedTasks={groupedTasks} />
		</div>
	);
};
