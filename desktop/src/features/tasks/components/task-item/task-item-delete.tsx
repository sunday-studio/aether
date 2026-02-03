import { useQueryClient } from "@tanstack/react-query";
import { Trash } from "lucide-react";

import { Tooltip } from "~/components/shared/tooltip";
import { cn } from "~/utils/cn";
import { useDeleteTask } from "~/aether-sdk";
import { invalidateTaskQueries } from "../../invalidate-task-queries";
import { TaskActionButton } from "./task-shared-components";

export const TaskItemDelete = ({ taskId, goalId }: { taskId: string, goalId?: string }) => {
	const queryClient = useQueryClient();
	const { mutate: deleteTask } = useDeleteTask();

	const handleClick = () => {
		deleteTask(
			{ id: taskId },
			{
				onSuccess: () => invalidateTaskQueries(queryClient, { goalId }),
			},
		);
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
