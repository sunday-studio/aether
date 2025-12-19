/** biome-ignore-all lint/a11y/noStaticElementInteractions: <explanation> */
import { useQueryClient } from "@tanstack/react-query";
import { format } from "date-fns";
import { getGetInboxTasksQueryKey, useUpdateTask } from "~/aether-sdk";
import type { DbTask } from "~/aether-sdk/models";
import { useOptimisticUpdateTask } from "../../use-optimistic-update-task";
import { TaskItemCheckbox } from "./task-item-checkbox";
import { TaskDescriptionInput } from "./task-item-description";
import { TaskTitleInput } from "./task-item-title";

interface TaskItemProps {
	task: DbTask;
}

interface TaskInputProps {
	value: string | undefined;
	onChange: (value: string) => void;
}

// const TaskDueDateInput = ({ value, onChange }: TaskInputProps) => {
// 	if (!value)
// 		return (
// 			<p className="p-0.5 rounded-sm bg-neutral-100 text-neutral-500 text-sm">
// 				Add due date
// 			</p>
// 		);
// 	return (
// 		<input
// 			type="date"
// 			className="w-40 text-sm"
// 			value={value ? format(value, "yyyy-MM-dd") : ""}
// 			onChange={(e) => onChange(e.target.value)}
// 		/>
// 	);
// };

export const TaskItem = ({ task }: TaskItemProps) => {
	const { mutate: updateTask } = useOptimisticUpdateTask();

	const handleOnUpdateTask = (
		inputName: string,
		inputValue: string | boolean | undefined,
	) => {
		updateTask({
			id: task.id as string,
			data: {
				[inputName]: inputValue,
			},
		});
	};

	return (
		<div className="flex gap-4 w-full overflow-hidden py-1 px-1">
			<div className="flex items-start mt-1">
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
				<div className="flex items-center gap-1">
					{/* <TaskDueDateInput
						value={task.dueDate}
						onChange={(value) => {
							handleOnUpdateTask("dueDate", new Date(value).toISOString());
						}}
					/> */}

					{/* TODO: come back to this later */}
					{/* <TaskTagsInput
						taskId={task.id as string}
						value={task.tags ?? []}
						// value={task.tags.map((tag) => tag.name).join(", ")}
						onChange={(value) => {
							handleOnUpdateTask("tagIds", value);
						}}
					/> */}
				</div>
			</div>
		</div>
	);
};
