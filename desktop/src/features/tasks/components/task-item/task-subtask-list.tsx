import { useQueryClient } from "@tanstack/react-query";
import { Flag } from "lucide-react";
import { Button } from "react-aria-components";
import { useCreateSubtask } from "~/aether-sdk";
import { Tooltip } from "~/components/shared/tooltip";
import { invalidateSubtaskQueries, invalidateTaskQueries } from "../../invalidate-task-queries";
import { TaskActionButton } from "./task-shared-components";

export const TaskSubtasksTrigger = ({
	taskId,
	goalId,
}: {
	taskId: string;
	goalId?: string;
}) => {
	const { mutate: createSubtask } = useCreateSubtask();
	const queryClient = useQueryClient();

	const handleCreateSubtask = () => {
		createSubtask(
			{
				taskId,
				data: { title: "New Subtask" },
			},
			{
				onSuccess: () => {
					invalidateSubtaskQueries(queryClient, taskId);
					invalidateTaskQueries(queryClient, { goalId });
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
