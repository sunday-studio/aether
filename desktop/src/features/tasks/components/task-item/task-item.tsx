import { useQueryClient } from "@tanstack/react-query";
import { useUpdateTask } from "~/aether-sdk";
import type { Tag, TaskWithSubtasks } from "~/aether-sdk/models";
import { convertCalendarDateToIsoString } from "~/utils/date";
import { invalidateTaskQueries } from "../../invalidate-task-queries";
import { SubtaskList } from "../sub-task-item/subtask-list";
import { TaskGoalSelector } from "./task-goal-selector";
import { TaskItemCheckbox } from "./task-item-checkbox";
import { TaskItemDelete } from "./task-item-delete";
import { TaskDescriptionInput } from "./task-item-description";
import { TaskDueDateInput } from "./task-item-due-date";
import { TaskTitleInput } from "./task-item-title";
import { TaskSubtasksTrigger } from "./task-subtask-list";
import { TaskTagsInput } from "./task-tags-selector";

interface TaskItemProps {
	task: TaskWithSubtasks;
}

const Divider = () => {
	return <span className="text-xs text-neutral-400">•</span>;
};

export const TaskItem = ({ task }: TaskItemProps) => {
	const queryClient = useQueryClient();
	const { mutate: updateTask } = useUpdateTask();

	const handleOnUpdateTask = (
		inputName: string,
		inputValue: string | boolean | null,
	) => {
		updateTask(
			{
				id: task.id as string,
				data: {
					[inputName]: inputValue,
				},
			},
			{
				onSuccess: () =>
					invalidateTaskQueries(queryClient, {
						goalId: task.goalId ?? undefined,
					}),
			},
		);
	};

	return (
		<div className="flex gap-4 w-full overflow-hidden  pb-1">
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
					value={task.description ?? null}
					onChange={(value) => {
						handleOnUpdateTask("description", value);
					}}
				/>
				<SubtaskList
					taskId={task.id as string}
					subtasks={task.subtasks}
					goalId={task.goalId ?? undefined}
				/>
				<div className="flex gap-1 items-center">
					<TaskDueDateInput
						value={task.dueDate ?? undefined}
						onChange={(value) => {
							handleOnUpdateTask(
								"dueDate",
								value ? convertCalendarDateToIsoString(value) : null,
							);
						}}
					/>
					<Divider />
					<TaskTagsInput
						taskId={task.id as string}
						value={(task as TaskWithSubtasks & { tags?: Tag[] }).tags}
					/>
					<Divider />
					<TaskGoalSelector value={task?.goalId} taskId={task.id as string} />
					<Divider />
					<TaskSubtasksTrigger
						taskId={task.id as string}
						goalId={task.goalId ?? undefined}
					/>
					<Divider />
					<TaskItemDelete
						taskId={task.id as string}
						goalId={task.goalId ?? undefined}
					/>
				</div>
			</div>
		</div>
	);
};
