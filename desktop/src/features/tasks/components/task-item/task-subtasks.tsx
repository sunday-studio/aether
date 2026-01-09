import { useQueryClient } from "@tanstack/react-query";
import { Circle, CircleDashed, GripHorizontal, Loader2 } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import {
	getGetGoalInstancesQueryKey,
	getGetInboxTasksQueryKey,
	getGetOverdueTasksQueryKey,
	getGetSubTasksQueryKey,
	useCreateSubTask,
	useGetSubTasks,
} from "~/aether-sdk";
import type { DbSubTask } from "~/aether-sdk/models/db-sub-task";
import { useDebounceCallback } from "~/hooks/use-debounce";
import { useOptimisticUpdateSubtask } from "../../use-optimistic-task-hooks";

interface TaskSubtasksProps {
	taskId: string;
	goalId?: string;
}

interface TaskSubtaskItemProps {
	subtask: DbSubTask;
	onChangeTitleChange: (value: string) => void;
	onChangeIsCompletedChange: (value: boolean) => void;
	onKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void;
	setInputRef: (el: HTMLInputElement | null) => void;
}

const TaskSubtaskItem = ({
	subtask,
	onChangeTitleChange,
	onChangeIsCompletedChange,
	onKeyDown,
	setInputRef,
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
		<div className="flex gap-2 items-center border-b border-neutral-200 group first:border-t hover:bg-neutral-100 [&:has(input:focus)]:bg-neutral-100 [&:has(input:focus)]:border-neutral-300">
			<button
				type="button"
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
				className="text-[13px] w-full h-full py-1.5 border-0 outline-0 bg-transparent"
				onChange={handleChange}
				onKeyDown={onKeyDown}
			/>

			<div className="p-0.5 rounded-sm flex items-center justify-center text-neutral-400 -ml-5 transition-transform duration-200 group-hover:bg-neutral-200 cursor-pointer">
				<GripHorizontal size={15} strokeWidth={2} />
			</div>
		</div>
	);
};

export const TaskSubtasks = ({ taskId, goalId }: TaskSubtasksProps) => {
	const { mutate: updateSubtask } = useOptimisticUpdateSubtask();
	const { mutate: createSubtask } = useCreateSubTask();
	const { data: subtasksData, isLoading } = useGetSubTasks(taskId);

	const subtasks: DbSubTask[] = (subtasksData?.data as DbSubTask[]) ?? [];

	const inputRefs = useRef<Map<string, HTMLInputElement>>(new Map());
	const pendingFocusRef = useRef<string | null>(null);

	// Callback ref to store input element and handle focusing
	const setInputRef = (subtaskId: string) => (el: HTMLInputElement | null) => {
		if (el) {
			inputRefs.current.set(subtaskId, el);

			// If this is the subtask we want to focus, focus it now
			if (pendingFocusRef.current === subtaskId) {
				requestAnimationFrame(() => {
					el.focus();
					el.select();
					pendingFocusRef.current = null;
				});
			}
		} else {
			inputRefs.current.delete(subtaskId);
		}
	};

	// Helper function to get input element
	const getInputElement = (subtaskId: string) => {
		return inputRefs.current.get(subtaskId);
	};

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

	const queryClient = useQueryClient();
	const subtasksQueryKey = getGetSubTasksQueryKey(taskId);
	const inboxTasksQueryKey = getGetInboxTasksQueryKey();
	const overdueTasksQueryKey = getGetOverdueTasksQueryKey();

	const handleCreateSubtask = (title: string) => {
		const newTitle = title || "New Subtask";

		createSubtask(
			{
				taskId: taskId,
				data: {
					title: newTitle,
				},
			},
			{
				onSuccess: async (response: unknown) => {
					const newSubtask = response as DbSubTask;
					const subtaskId = newSubtask.id as string;

					// Refetch queries and wait for completion
					await Promise.all([
						queryClient.refetchQueries({ queryKey: subtasksQueryKey }),
						queryClient.invalidateQueries({ queryKey: inboxTasksQueryKey }),
						queryClient.invalidateQueries({ queryKey: overdueTasksQueryKey }),
						goalId
							? queryClient.invalidateQueries({
									queryKey: getGetGoalInstancesQueryKey(goalId),
								})
							: Promise.resolve(),
					]);

					// Set the pending focus ref - the callback ref will handle focusing when element mounts
					pendingFocusRef.current = subtaskId;

					// Also try to focus immediately in case ref is already set
					setTimeout(() => {
						const input = getInputElement(subtaskId);
						if (input && pendingFocusRef.current === subtaskId) {
							requestAnimationFrame(() => {
								input.focus();
								input.select();
								pendingFocusRef.current = null;
							});
						}
					}, 150);
				},
			},
		);
	};

	const handleKeyDown = (
		e: React.KeyboardEvent<HTMLInputElement>,
		_currentSubtaskId: string,
		currentIndex: number,
	) => {
		if (e.key === "Enter") {
			e.preventDefault();
			// Always create a new subtask when Enter is pressed
			handleCreateSubtask("");
		} else if (e.key === "ArrowDown") {
			e.preventDefault();
			const nextIndex = currentIndex + 1;
			if (nextIndex < subtasks.length && subtasks[nextIndex]) {
				const nextSubtaskId = subtasks[nextIndex]?.id as string;
				if (nextSubtaskId) {
					const nextInput = getInputElement(nextSubtaskId);
					if (nextInput) {
						nextInput.focus();
					}
				}
			}
		} else if (e.key === "ArrowUp") {
			e.preventDefault();
			const prevIndex = currentIndex - 1;
			if (prevIndex >= 0 && subtasks[prevIndex]) {
				const prevSubtaskId = subtasks[prevIndex]?.id as string;
				if (prevSubtaskId) {
					const prevInput = getInputElement(prevSubtaskId);
					if (prevInput) {
						prevInput.focus();
					}
				}
			}
		}
	};

	if (isLoading) {
		return (
			<div className="flex flex-col my-3">
				<div className="flex gap-2 items-center border-b border-neutral-200 first:border-t">
					<div className="w-4 h-4 flex items-center justify-center">
						<Loader2
							size={12}
							strokeWidth={2}
							className="animate-spin text-neutral-400"
						/>
					</div>
					<div className="text-[13px] text-neutral-400 py-1.5">
						Loading subtasks...
					</div>
				</div>
			</div>
		);
	}

	if (subtasks.length === 0) return null;

	return (
		<div className="flex  flex-col my-3">
			{subtasks.map((subtask, index) => {
				const subtaskId = subtask.id as string;
				return (
					<TaskSubtaskItem
						key={subtaskId}
						subtask={subtask}
						setInputRef={setInputRef(subtaskId)}
						onChangeTitleChange={(value) =>
							handleOnChangeTitleChange(subtaskId, value)
						}
						onChangeIsCompletedChange={(value) => {
							handleOnChangeIsCompletedChange(subtaskId, value);
						}}
						onKeyDown={(e) => handleKeyDown(e, subtaskId, index)}
					/>
				);
			})}
		</div>
	);
};
