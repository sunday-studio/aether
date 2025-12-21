import { useQueryClient } from "@tanstack/react-query";
import { isBefore, startOfDay } from "date-fns";
import {
	getGetInboxTasksQueryKey,
	useCreateTask,
	useGetInboxTasks,
} from "~/aether-sdk";
import { AddNewButton } from "~/components/shared/button";
import { OverdueTasks } from "./components/overdue-tasks";
import { TasksContainer } from "./components/task-item/tasks-container";
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
				onSuccess: () => {
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
					const isPast = isBefore(
						startOfDay(new Date(date)),
						startOfDay(new Date()),
					);
					return (
						<TasksContainer
							key={date}
							date={date}
							tasks={tasks}
							isPast={isPast}
						/>
					);
				})}
			</ul>
		</div>
	);
};
