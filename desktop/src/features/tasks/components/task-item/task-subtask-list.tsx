import { Flag } from "lucide-react";
import { Tooltip } from "~/components/shared/tooltip";
import { TaskActionButton } from "./task-shared-components";
import { useCreateSubTask } from "~/aether-sdk";
import { Button } from "react-aria-components";

export const TaskSubtasksTrigger = ({ taskId }: { taskId: string }) => {
	const { mutate: createSubtask } = useCreateSubTask();


	const handleCreateSubtask = () => {
		createSubtask({
			taskId: taskId,
			data: {
				title: "New Subtask",
			},
		});
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
