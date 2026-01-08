import { Trash } from "lucide-react";

import { Tooltip } from "~/components/shared/tooltip";
import { cn } from "~/utils/cn";
import { useOptimisticDeleteTask } from "../../use-optimistic-task-hooks";
import { TaskActionButton } from "./task-shared-components";

export const TaskItemDelete = ({ taskId, goalId }: { taskId: string, goalId?: string }) => {
	const { mutate: deleteTask } = useOptimisticDeleteTask();

	const handleClick = () => {
		deleteTask({ id: taskId, goalId });
	};

	return (
		<Tooltip
			content="Delete task"
			trigger={
				<button
					type="button"
					onClick={handleClick}
					className="touch-none select-none"
				>
					<TaskActionButton
						className={cn("hover:text-red-400 hover:bg-red-500/20")}
					>
						<Trash size={14} strokeWidth={3} />
					</TaskActionButton>
				</button>
			}
		/>
	);
};
