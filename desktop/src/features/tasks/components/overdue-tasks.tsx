import { useGetOverdueTasks } from "~/aether-sdk";
import { TaskItem } from "./task-item/task-item";
import { TaskListDivider } from "./task-list-divider";

export const OverdueTasks = () => {
	const { data: overdueTasks } = useGetOverdueTasks();

	const hasOverDueTasks =
		overdueTasks?.data?.length && overdueTasks.data.length > 0;

	if (!hasOverDueTasks) return null;

	return (
		<div className="px-0.5">
			<TaskListDivider isOverdue={true} date={undefined} />
			<ul className="w-full h-full overflow-y-scroll">
				{overdueTasks?.data?.slice(0, 10).map((task) => (
					<TaskItem key={task.id} task={task} />
				))}
			</ul>
		</div>
	);
};
