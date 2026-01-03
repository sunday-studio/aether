import type { DbTask } from "~/aether-sdk/models";
import { useOptimisticUpdateTask } from "../../use-optimistic-update-task";
import { TaskGoalSelector } from "./task-goal-selector";
import { TaskItemCheckbox } from "./task-item-checkbox";
import { TaskDescriptionInput } from "./task-item-description";
import { TaskDueDateInput } from "./task-item-due-date";
import { TaskTitleInput } from "./task-item-title";
import { TaskSubtasksTrigger } from "./task-subtask-list";
import { TaskTagsInput } from "./task-tags-selector";

interface TaskItemProps {
	task: DbTask;
}

export const TaskItem = ({ task }: TaskItemProps) => {
	const { mutate: updateTask } = useOptimisticUpdateTask();

	const handleOnUpdateTask = (
		inputName: string,
		inputValue: string | boolean | null,
	) => {
		updateTask({
			id: task.id as string,
			data: {
				[inputName]: inputValue,
			},
		});
	};

	return (
		<div className="flex gap-4 w-full  overflow-hidden">
			<div className="flex items-start">
				<TaskItemCheckbox
					isChecked={task.isCompleted ?? false}
					onChange={(isChecked) => {
						handleOnUpdateTask("isCompleted", isChecked);
					}}
				/>
			</div>
			<div className="flex-1 flex flex-col gap-1.5">
				<TaskTitleInput
					value={task.title}
					onChange={(value) => {
						handleOnUpdateTask("title", value);
					}}
				/>
				<TaskDescriptionInput
					value={task.description}
					onChange={(value) => {
						handleOnUpdateTask("description", value);
					}}
				/>
				<div className="flex gap-1 items-center">
					<TaskDueDateInput
						value={task.dueDate}
						onChange={(value) => {
							let date = null;
							if (value) {
								const now = new Date();
								const tempDate = value.toDate("UTC");
								tempDate.setUTCHours(
									now.getUTCHours(),
									now.getUTCMinutes(),
									now.getUTCSeconds(),
									now.getUTCMilliseconds(),
								);
								date = tempDate.toISOString();
							}

							handleOnUpdateTask("dueDate", date);
						}}
					/>

					<p className="text-xs text-neutral-400">•</p>
					<TaskTagsInput taskId={task.id as string} value={task.tags ?? []} />
					<p className="text-xs text-neutral-400">•</p>
					<TaskGoalSelector
						value={task.goalInstanceId}
						taskId={task.id as string}
					/>
					<p className="text-xs text-neutral-400">•</p>
					<TaskSubtasksTrigger />
				</div>
			</div>
		</div>
	);
};
