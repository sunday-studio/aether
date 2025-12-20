import { useQueryClient } from "@tanstack/react-query";
import {
	getGetInboxTasksQueryKey,
	useCreateTask,
	useGetInboxTasks,
} from "~/aether-sdk";
import { AddNewButton } from "~/components/shared/button";
import { OverdueTasks } from "./components/overdue-tasks";
import { TaskItem } from "./components/task-item/task-item";
import { TaskListDivider } from "./components/task-list-divider";
import { groupTaskByCreatedAt } from "./tasks.domain";

export const InboxTasksView = () => {
	const queryClient = useQueryClient();
	const inboxTasksQueryKey = getGetInboxTasksQueryKey();
	const { data: inboxTasks } = useGetInboxTasks();

	const { mutate: createTask } = useCreateTask();

	const groupedTasks = groupTaskByCreatedAt(inboxTasks?.data ?? []);

	const handleCreateTask = () => {
		createTask(
			{
				data: {
					title: "New Task",
				},
			},
			{
				onSuccess: ({ data }) => {
					queryClient.invalidateQueries({ queryKey: inboxTasksQueryKey });
				},
			},
		);
	};

	return (
		<div className="h-full">
			<div className="flex items-center justify-between py-4">
				<h3 className="newsreader-font text-2xl font-medium">Inbox</h3>
				<AddNewButton
					onClick={handleCreateTask}
					label="Add task"
					shortcuts={["⌘", "N"]}
				/>
			</div>
			<ul className="w-full h-full overflow-y-scroll">
				<OverdueTasks />
				{Object.entries(groupedTasks).map(([date, tasks]) => {
					return (
						<li key={date} className="space-y-4 px-0.5">
							<TaskListDivider date={date} />
							{tasks.map((task) => (
								<TaskItem key={task.id} task={task} />
							))}
						</li>
					);
				})}
			</ul>
		</div>
	);
};
