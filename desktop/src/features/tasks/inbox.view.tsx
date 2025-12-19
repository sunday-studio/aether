import { useQueryClient } from "@tanstack/react-query";
import {
	getGetInboxTasksQueryKey,
	useCreateTask,
	useGetInboxTasks,
} from "~/aether-sdk";
import { TaskItem } from "./components/task-item/task-item";
import { TaskListDivider } from "./components/task-list-divider";
import { groupTaskByCreatedAt } from "./tasks.domain";

export const InboxTasksView = () => {
	const queryClient = useQueryClient();
	const inboxTasksQueryKey = getGetInboxTasksQueryKey();
	const { data: inboxTasks } = useGetInboxTasks();

	console.log("inboxTasks", inboxTasks);

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
					console.log(data);
					queryClient.invalidateQueries({ queryKey: inboxTasksQueryKey });
				},
			},
		);
	};

	return (
		<div className="h-full">
			<div className="flex items-center justify-between py-4">
				<p>Inbox</p>
				<button type="button" onClick={handleCreateTask}>
					Add Task
				</button>
			</div>
			<ul className="w-full h-full overflow-y-scroll">
				{Object.entries(groupedTasks).map(([date, tasks]) => {
					return (
						<li key={date} className="space-y-4">
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
