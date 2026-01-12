import { Circle, CircleDashed, GripHorizontal, Trash2 } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import type { DbSubTask } from "~/aether-sdk/models/db-sub-task";
import { Tooltip } from "~/components/shared/tooltip";
import { useDebounceCallback } from "~/hooks/use-debounce";

interface TaskSubtaskItemProps {
	subtask: DbSubTask;
	onChangeTitleChange: (value: string) => void;
	onChangeIsCompletedChange: (value: boolean) => void;
	onKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void;
	setInputRef: (el: HTMLInputElement | null) => void;
	onDelete: () => void;
}

export const TaskSubtaskItem = ({
	subtask,
	onChangeTitleChange,
	onChangeIsCompletedChange,
	onKeyDown,
	setInputRef,
	onDelete,
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

	// Set the ref callback
	useEffect(() => {
		setInputRef(inputRef.current);
		return () => setInputRef(null);
	}, [setInputRef]);

	const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
		const newValue = e.target.value;
		setInputValue(newValue);
		debouncedOnChangeTitleChange(newValue);
	};

	return (
		<div className="px-1 flex gap-2 items-center border-b border-neutral-200 group first:border-t hover:bg-neutral-100 [&:has(input:focus)]:bg-neutral-100 [&:has(input:focus)]:border-neutral-300">
			<button
				type="button"
				className="w-4 h-4 flex items-center justify-center text-neutral-500 cursor-pointer"
				onClick={() => {
					onChangeIsCompletedChange(!subtask.isCompleted);
				}}
			>
				{subtask.isCompleted ? (
					<Circle
						size={12}
						strokeWidth={2.5}
						className="text-green-600 cursor-pointer"
					/>
				) : (
					<CircleDashed size={12} strokeWidth={2.5} />
				)}
			</button>
			<input
				type="text"
				value={inputValue}
				ref={inputRef}
				className="text-[13px] w-full h-full py-1.5 border-0 outline-0 bg-transparent"
				onChange={handleChange}
				onKeyDown={onKeyDown}
				placeholder="Untitled"
			/>

			<div className="flex items-center gap-1">
				{/* TODO: do the ordering functionality later */}
				{/* <div className="p-0.5 opacity-0 group-hover:opacity-100 hover:bg-neutral-200 rounded-sm flex items-center justify-center text-neutral-400  transition-transform duration-200 cursor-pointer">
					<GripHorizontal size={15} strokeWidth={2} />
				</div> */}

				<button
					type="button"
					tabIndex={0}
					className="p-0.5 opacity-0 group-hover:opacity-100 hover:bg-neutral-200 rounded-sm flex items-center justify-center text-neutral-400 transition-transform duration-200 cursor-pointer"
					onClick={onDelete}
				>
					<Trash2 size={13} strokeWidth={2} />
				</button>
			</div>
		</div>
	);
};
