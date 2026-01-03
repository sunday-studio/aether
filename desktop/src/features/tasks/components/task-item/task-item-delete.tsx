import { Trash } from "lucide-react";
import { useState } from "react";
import { Button } from "react-aria-components";
import { useDeleteTaskById } from "~/aether-sdk";
import { showToast } from "~/components/shared/toast-components";
import { Tooltip } from "~/components/shared/tooltip";
import { cn } from "~/utils/cn";
import { TaskActionButton } from "./task-shared-components";

export const TaskItemDelete = ({ taskId }: { taskId: string }) => {
	const [isFirstClick, setIsFirstClick] = useState(true);
	const { mutate: deleteTask } = useDeleteTaskById();

	const handleOnDeleteTask = () => {
		setIsFirstClick(false);

		if (isFirstClick) {
			return;
		}

		deleteTask(
			{ id: taskId },
			{
				onSuccess: () => {
					showToast({
						title: "Task deleted successfully",
					});
				},
			},
		);
	};

	return (
		<Tooltip
			content={isFirstClick ? "Delete task" : "Confirm delete task"}
			trigger={
				<Button onClick={handleOnDeleteTask}>
					<TaskActionButton
						className={cn("hover:text-red-400 hover:bg-red-500/20")}
					>
						<Trash size={14} strokeWidth={3} />
					</TaskActionButton>
				</Button>
			}
		/>
		// <div>
		// 	<Button>
		// 		<Trash2 className="size-4" />
		// 	</Button>
		// </div>
	);
};
