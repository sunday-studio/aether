import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useRef } from "react";
import type { SubTask } from "~/aether-sdk/models/sub-task";
import {
	useCreateSubtask,
	useDeleteSubtask,
	useUpdateSubtask,
} from "~/aether-sdk";
import {
	invalidateSubtaskQueries,
	invalidateTaskQueries,
} from "../../invalidate-task-queries";
import { TaskSubtaskItem } from "./subtask-item";

interface SubtaskListProps {
	taskId: string;
	subtasks: SubTask[];
	goalId?: string;
}

export const SubtaskList = ({ taskId, subtasks, goalId }: SubtaskListProps) => {
	const queryClient = useQueryClient();
	const { mutate: updateSubtask } = useUpdateSubtask();
	const { mutate: createSubtask } = useCreateSubtask();
	const { mutate: deleteSubtask } = useDeleteSubtask();

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

	const invalidate = useCallback(() => {
		invalidateSubtaskQueries(queryClient, taskId);
		invalidateTaskQueries(queryClient, { goalId });
	}, [queryClient, taskId, goalId]);

	const handleOnChangeTitleChange = (subtaskId: string, value: string) => {
		updateSubtask(
			{
				taskId: taskId,
				subtaskId: subtaskId,
				data: { title: value },
			},
			{ onSuccess: invalidate },
		);
	};

	const handleOnChangeIsCompletedChange = (
		subtaskId: string,
		value: boolean,
	) => {
		updateSubtask(
			{
				taskId: taskId,
				subtaskId: subtaskId,
				data: { isCompleted: value },
			},
			{ onSuccess: invalidate },
		);
	};

	const handleCreateSubtask = (title: string) => {
		createSubtask(
			{
				taskId: taskId,
				data: { title },
			},
			{
				onSuccess: ({ data: newSubtask }) => {
					invalidate();
					const newId = newSubtask?.id as string;
					if (!newId) return;
					pendingFocusRef.current = newId;
					// Wait for task list refetch so new subtask is in DOM, then focus
					setTimeout(() => {
						const input = getInputElement(newId);
						if (input && pendingFocusRef.current === newId) {
							requestAnimationFrame(() => {
								input.focus();
								input.select();
								pendingFocusRef.current = null;
							});
						}
					}, 400);
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
							deleteSubtask(
								{ taskId, subtaskId },
								{ onSuccess: invalidate },
							)
						}
					/>
				);
			})}
		</div>
	);
};
