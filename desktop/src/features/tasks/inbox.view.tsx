import { useQueryClient } from "@tanstack/react-query";
import {
	getGetInboxTasksQueryKey,
	useCreateTask,
	useGetInboxTasks,
} from "~/aether-sdk";
import { AddNewButton } from "~/components/shared/button";
import { VirtualizedTaskList } from "./components/virtualized-task-list";
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
		<div className="h-full flex flex-col">
			<div className="flex items-center justify-between py-4">
				<h3 className="newsreader-font text-2xl font-medium">Inbox</h3>
				<AddNewButton
					onClick={handleCreateTask}
					label="Add task"
					shortcuts={["⌘", "N"]}
				/>
			</div>
			<VirtualizedTaskList groupedTasks={groupedTasks} />
		</div>
	);
};
