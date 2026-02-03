import { useCallback, useRef } from "react";
import type { SubTask } from "~/aether-sdk/models/sub-task";
import {
	useOptimisticCreateSubtask,
	useOptimisticDeleteSubtask,
	useOptimisticUpdateSubtask,
} from "../../use-optimistic-task-hooks";
import { TaskSubtaskItem } from "./subtask-item";

interface SubtaskListProps {
	taskId: string;
	subtasks: SubTask[];
}

export const SubtaskList = ({ taskId, subtasks }: SubtaskListProps) => {
	const { mutate: updateSubtask } = useOptimisticUpdateSubtask();
	const { mutate: createSubtask } = useOptimisticCreateSubtask();
	const { mutate: deleteSubtask } = useOptimisticDeleteSubtask();

	const inputRefs = useRef<Map<string, HTMLInputElement>>(new Map());
	const pendingFocusRef = useRef<string | null>(null);

	// Use useCallback to stabilize the ref callback and prevent unnecessary cleanup
	const setInputRef = useCallback(
		(subtaskId: string) => (el: HTMLInputElement | null) => {
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
				// Only delete the ref if the subtask no longer exists in the list
				// This prevents cleanup from running during re-renders when subtasks still exist
				const subtaskExists = subtasks?.some((s) => s.id === subtaskId);
				if (!subtaskExists) {
					inputRefs.current.delete(subtaskId);
				}
			}
		},
		[subtasks], // Only recreate when subtasks data changes
	);

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

	const handleCreateSubtask = (title: string) => {
		createSubtask(
			{
				taskId: taskId,
				data: {
					title: title,
				},
			},
			{
				onSuccess: ({ data: newSubtask }: { data: DbSubTask }) => {
					const subtaskId = newSubtask.id as string;

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
						onDelete={() =>
							deleteSubtask({ taskId: taskId, subtaskId: subtaskId })
						}
					/>
				);
			})}
		</div>
	);
};
