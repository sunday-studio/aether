import { Flag } from "lucide-react";
import { Tooltip } from "~/components/shared/tooltip";
import { TaskActionButton } from "./task-shared-components";

export const TaskSubtaskList = () => {
	return (
		<div>
			<h1>Task Subtask List</h1>
		</div>
	);
};

export const TaskSubtasksTrigger = () => {
	return (
		<Tooltip
			trigger={
				<TaskActionButton>
					<Flag size={14} strokeWidth={3} />
				</TaskActionButton>
			}
			content="Add sub-tasks"
		/>
	);
};
