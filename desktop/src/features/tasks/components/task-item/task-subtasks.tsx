import { useQueryClient } from "@tanstack/react-query";
import {
	ArrowRight,
	Circle,
	CircleDashed,
	GripHorizontal,
	PlusIcon,
	Tally3,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Button } from "react-aria-components";
import { useUpdateSubTask } from "~/aether-sdk";
import type { DbSubTask } from "~/aether-sdk/models/db-sub-task";
import { useDebounceCallback } from "~/hooks/use-debounce";

interface TaskSubtasksProps {
	taskId: string;
	subtasks: DbSubTask[];
}

interface TaskSubtaskItemProps {
	subtask: DbSubTask;
	onChangeTitleChange: (value: string) => void;
	onChangeIsCompletedChange: (value: boolean) => void;
}

const TaskSubtaskItem = ({
	subtask,
	onChangeTitleChange,
	onChangeIsCompletedChange,
}: TaskSubtaskItemProps) => {
	const [inputValue, setInputValue] = useState(subtask.title ?? "");
	const inputRef = useRef<HTMLInputElement>(null);
	const debouncedOnChangeTitleChange = useDebounceCallback(
		onChangeTitleChange,
		500,
	);

	// Sync with external value changes when input is not focused
	useEffect(() => {
		const input = inputRef.current;
		if (!input) return;

		// Don't update if the input is focused (user is typing)
		if (document.activeElement === input) return;

		// Only update if value actually changed
		if (subtask.title !== inputValue) {
			setInputValue(subtask.title ?? "");
		}
	}, [inputValue, subtask.title]);

	const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
		const newValue = e.target.value;
		setInputValue(newValue);
		debouncedOnChangeTitleChange(newValue);
	};

	return (
		<div className="flex gap-2 items-center border-b border-neutral-200 group first:border-t hover:bg-neutral-100">
			<button
				className="w-4 h-4 flex items-center justify-center text-neutral-500 cursor-pointer"
				onClick={() => {
					onChangeIsCompletedChange(!subtask.isCompleted);
				}}
			>
				{subtask.isCompleted ? (
					<Circle
						size={15}
						strokeWidth={2}
						className="text-green-600 cursor-pointer"
					/>
				) : (
					<CircleDashed size={15} strokeWidth={2} />
				)}
			</button>
			<input
				type="text"
				value={inputValue}
				ref={inputRef}
				className="text-[13px] w-full h-full py-1.5 border-0 outline-0 bg-transparent "
				onChange={handleChange}
			/>

			<div className="p-0.5 rounded-sm flex items-center justify-center text-neutral-400 -ml-5 transition-transform duration-200 group-hover:bg-neutral-200 cursor-pointer ">
				<GripHorizontal size={15} strokeWidth={2} />
			</div>
		</div>
	);
};

export const TaskSubtasks = ({ taskId, subtasks }: TaskSubtasksProps) => {
	const { mutate: updateSubtask } = useUpdateSubTask();

	const handleOnChangeTitleChange = (subtaskId: string, value: string) => {
		updateSubtask({
			taskId: taskId,
			subtaskId: subtaskId,
			data: {
				title: value,
			},
		});
	};

	const handleOnChangeIsCompletedChange = (
		subtaskId: string,
		value: boolean,
	) => {
		updateSubtask({
			taskId: taskId,
			subtaskId: subtaskId,
			data: {
				isCompleted: value,
			},
		});
	};
	return (
		<div className="flex  flex-col my-3">
			{subtasks.map((subtask) => (
				<TaskSubtaskItem
					key={subtask.id}
					subtask={subtask}
					onChangeTitleChange={(value) =>
						handleOnChangeTitleChange(subtask.id as string, value)
					}
					onChangeIsCompletedChange={(value) => {
						handleOnChangeIsCompletedChange(subtask.id as string, value);
					}}
				/>
			))}
		</div>
	);
};
