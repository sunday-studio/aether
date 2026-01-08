import { Trash } from "lucide-react";

import { Tooltip } from "~/components/shared/tooltip";
import { cn } from "~/utils/cn";
import { useOptimisticDeleteTask } from "../../use-optimistic-update-task";
import { TaskActionButton } from "./task-shared-components";

export const TaskItemDelete = ({ taskId }: { taskId: string }) => {
	const { mutate: deleteTask } = useOptimisticDeleteTask();

	const handleClick = () => {
		deleteTask({ id: taskId });
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
