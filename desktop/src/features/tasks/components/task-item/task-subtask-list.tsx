import { useQueryClient } from "@tanstack/react-query";
import { Flag } from "lucide-react";
import { Button } from "react-aria-components";
import { getGetSubTasksQueryKey, useCreateSubTask } from "~/aether-sdk";
import { Tooltip } from "~/components/shared/tooltip";
import { TaskActionButton } from "./task-shared-components";

export const TaskSubtasksTrigger = ({ taskId }: { taskId: string }) => {
	const { mutate: createSubtask } = useCreateSubTask();
	const queryClient = useQueryClient();

	const handleCreateSubtask = () => {
		createSubtask(
			{
				taskId: taskId,
				data: {
					title: "New Subtask",
				},
			},
			{
				onSuccess: () => {
					queryClient.invalidateQueries({
						queryKey: getGetSubTasksQueryKey(taskId),
					});
				},
			},
		);
	};
	return (
		<Tooltip
			trigger={
				<Button onPress={handleCreateSubtask}>
					<TaskActionButton>
						<Flag size={14} strokeWidth={3} />
					</TaskActionButton>
				</Button>
			}
			content="Add sub-tasks"
		/>
	);
};
