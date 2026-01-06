import { useQueryClient } from "@tanstack/react-query";
import { Loader } from "lucide-react";
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
	const {
		data: inboxTasks,
		isLoading: isLoadingInboxTasks,
		error: errorInboxTasks,
	} = useGetInboxTasks();

	const { mutate: createTask } = useCreateTask();

	if (errorInboxTasks) {
		return (
			<div className="h-full flex items-center justify-center">
				<p className="text-sm text-neutral-500">Error loading inbox tasks</p>
			</div>
		);
	}

	if (isLoadingInboxTasks) {
		return (
			<div className="h-full flex items-center justify-center">
				<Loader className="w-4 h-4 animate-spin" />
			</div>
		);
	}

	console.log("inboxTasks ->", { inboxTasks, errorInboxTasks });

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
				<h3 className="font-gt-ultra text-2xl font-medium">Inbox</h3>
				<AddNewButton
					onClick={handleCreateTask}
					label="Add task"
					shortcuts={["⌘", "N"]}
				/>
			</div>
			{/* <VirtualizedTaskList groupedTasks={groupedTasks} /> */}
		</div>
	);
};
