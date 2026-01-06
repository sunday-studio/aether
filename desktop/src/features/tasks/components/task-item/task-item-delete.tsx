import { Trash } from "lucide-react";
import { useState } from "react";
import { Button } from "react-aria-components";
import { Tooltip } from "~/components/shared/tooltip";
import { cn } from "~/utils/cn";
import { useOptimisticDeleteTask } from "../../use-optimistic-update-task";
import { TaskActionButton } from "./task-shared-components";

export const TaskItemDelete = ({ taskId }: { taskId: string }) => {
	const [isFirstClick, setIsFirstClick] = useState(true);
	const { mutate: deleteTask } = useOptimisticDeleteTask();

	const handleOnDeleteTask = () => {
		setIsFirstClick(false);

		if (isFirstClick) {
			return;
		}

		deleteTask({ id: taskId });
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
	);
};
