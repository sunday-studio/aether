/** biome-ignore-all lint/a11y/noStaticElementInteractions: <explanation> */
import type { DbTask } from "~/aether-sdk/models";
import { useOptimisticUpdateTask } from "../../use-optimistic-update-task";
import { TaskItemCheckbox } from "./task-item-checkbox";
import { TaskDescriptionInput } from "./task-item-description";
import { TaskDueDateInput } from "./task-item-due-date";
import { TaskTitleInput } from "./task-item-title";
import { TaskTagsInput } from "./task-tags-field";

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
		<div className="flex gap-4 w-full overflow-hidden py-1 px-1 ">
			<div className="flex items-start mt-0.5">
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
				<div className="flex gap-1 items-center ">
					<TaskDueDateInput
						value={task.dueDate}
						onChange={(value) => {
							handleOnUpdateTask(
								"dueDate",
								value ? new Date(value).toISOString() : null,
							);
						}}
					/>
					<TaskTagsInput
						taskId={task.id as string}
						value={task.tags ?? []}
						// value={task.tags.map((tag) => tag.name).join(", ")}
						onChange={(value) => {
							handleOnUpdateTask("tagIds", value);
						}}
					/>
				</div>
			</div>
		</div>
	);
};
