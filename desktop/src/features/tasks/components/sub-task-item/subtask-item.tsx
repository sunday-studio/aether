import { Circle, CircleDashed, Trash2 } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import type { SubTask } from '~/aether-sdk/models';
import { useDebounceCallback } from '~/hooks/use-debounce';

interface TaskSubtaskItemProps {
	subtask: SubTask;
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
	const [inputValue, setInputValue] = useState(subtask.title ?? '');
	const inputRef = useRef<HTMLInputElement>(null);
	const debouncedOnChangeTitleChange = useDebounceCallback(onChangeTitleChange, 500);

	// Sync with external value changes when input is not focused
	useEffect(() => {
		const input = inputRef.current;
		if (!input) return;

		// Don't update if the input is focused (user is typing)
		if (document.activeElement === input) return;

		// Only update if value actually changed
		if (subtask.title !== inputValue) {
			setInputValue(subtask.title ?? '');
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
		<div className='group flex items-center gap-2 border-b border-neutral-200 px-1 first:border-t hover:bg-neutral-100 [&:has(input:focus)]:border-neutral-300 [&:has(input:focus)]:bg-neutral-100'>
			<button
				type='button'
				className='flex h-4 w-4 cursor-pointer items-center justify-center text-neutral-500'
				onClick={() => {
					onChangeIsCompletedChange(!subtask.isCompleted);
				}}
			>
				{subtask.isCompleted ? (
					<Circle size={12} strokeWidth={2.5} className='cursor-pointer text-green-600' />
				) : (
					<CircleDashed size={12} strokeWidth={2.5} />
				)}
			</button>
			<input
				type='text'
				value={inputValue}
				ref={inputRef}
				className='h-full w-full border-0 bg-transparent py-1.5 text-[12px] font-medium text-neutral-500 outline-0'
				onChange={handleChange}
				onKeyDown={onKeyDown}
				placeholder='Untitled'
			/>

			<div className='flex items-center gap-1'>
				<button
					type='button'
					tabIndex={0}
					className='flex cursor-pointer items-center justify-center rounded-sm p-0.5 text-neutral-400 opacity-0 transition-transform duration-200 group-hover:opacity-100 hover:bg-neutral-200'
					onClick={onDelete}
				>
					<Trash2 size={13} strokeWidth={2} />
				</button>
			</div>
		</div>
	);
};
