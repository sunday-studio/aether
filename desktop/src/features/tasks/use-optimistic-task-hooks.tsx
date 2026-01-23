import { useQueryClient } from "@tanstack/react-query";
import {
	getGetGoalInstancesQueryKey,
	getGetInboxTasksQueryKey,
	getGetOverdueTasksQueryKey,
	getGetSubtasksQueryKey,
	useCreateSubtask,
	useDeleteSubtask,
	useDeleteTask,
	useUpdateSubtask,
	useUpdateTask,
} from "~/aether-sdk";
import type { DbGoalInstance } from "~/aether-sdk/models/db-goal-instance";
import type { DbSubTask } from "~/aether-sdk/models/db-sub-task";

export const useOptimisticUpdateTask = () => {
	const queryClient = useQueryClient();
	const mutation = useUpdateTask();

	const inboxTasksQueryKey = getGetInboxTasksQueryKey();
	const { updateLocalInstance, previousTasks } = useOptimisticUpdateTaskQuery();

	const mutate = (variables: { id: string; data: Record<string, unknown> }) => {
		updateLocalInstance(variables);

		mutation.mutate(variables, {
			onError: () => {
				queryClient.setQueryData(inboxTasksQueryKey, previousTasks);
			},
		});
	};

	return {
		...mutation,
		mutate,
	};
};

/**
 * Optimistic update task query
 */
export const useOptimisticUpdateTaskQuery = () => {
	const queryClient = useQueryClient();
	const inboxTasksQueryKey = getGetInboxTasksQueryKey();
	const previousTasks = queryClient.getQueryData<{ data: DbTask[] }>(
		inboxTasksQueryKey,
	);

	const updateLocalInstance = (variables: {
		id: string;
		data: Record<string, unknown>;
	}) => {
		queryClient.setQueryData<{ data: DbTask[] }>(inboxTasksQueryKey, (old) => {
			const oldData = old?.data || [];

			const updatedData = oldData.map((task) =>
				task.id === variables.id ? { ...task, ...variables.data } : task,
			);

			return {
				...old,
				data: updatedData,
			};
		});
	};

	return {
		updateLocalInstance,
		previousTasks,
	};
};

/**
 * Optimistic delete task
 */
export const useOptimisticDeleteTask = () => {
	const queryClient = useQueryClient();
	const inboxTasksQueryKey = getGetInboxTasksQueryKey();
	const overdueTasksQueryKey = getGetOverdueTasksQueryKey();

	const mutation = useDeleteTask();

	const mutate = (variables: { id: string; goalId?: string }) => {
		// Store previous states for rollback
		const previousInboxTasks = queryClient.getQueryData(inboxTasksQueryKey);
		const previousOverdueTasks = queryClient.getQueryData(overdueTasksQueryKey);
		const previousGoalInstances = variables.goalId
			? queryClient.getQueryData(getGetGoalInstancesQueryKey(variables.goalId))
			: null;

		// Optimistically update inbox tasks
		queryClient.setQueryData<{ data: DbTask[] }>(inboxTasksQueryKey, (old) => {
			if (!old) return old;
			const oldData = old.data || [];
			return {
				...old,
				data: oldData.filter((task) => task.id !== variables.id),
			};
		});

		// Optimistically update overdue tasks
		queryClient.setQueryData<{ data: DbTask[] }>(
			overdueTasksQueryKey,
			(old) => {
				if (!old) return old;
				const oldData = old.data || [];
				return {
					...old,
					data: oldData.filter((task) => task.id !== variables.id),
				};
			},
		);

		// Optimistically update goal instances if goalId is provided
		if (variables.goalId) {
			const goalInstancesQueryKey = getGetGoalInstancesQueryKey(
				variables.goalId,
			);
			queryClient.setQueryData<{ data: DbGoalInstance[] }>(
				goalInstancesQueryKey,
				(old) => {
					if (!old) return old;
					const oldData = old.data || [];
					return {
						...old,
						data: oldData.map((goalInstance) => ({
							...goalInstance,
							tasks: (goalInstance.tasks || []).filter(
								(task) => task.id !== variables.id,
							),
						})),
					};
				},
			);
		}

		mutation.mutate(
			{ id: variables.id },
			{
				onError: () => {
					// Rollback to previous cache state if error
					if (previousInboxTasks) {
						queryClient.setQueryData(inboxTasksQueryKey, previousInboxTasks);
					}
					if (previousOverdueTasks) {
						queryClient.setQueryData(
							overdueTasksQueryKey,
							previousOverdueTasks,
						);
					}
					if (previousGoalInstances && variables.goalId) {
						const goalInstancesQueryKey = getGetGoalInstancesQueryKey(
							variables.goalId,
						);
						queryClient.setQueryData(
							goalInstancesQueryKey,
							previousGoalInstances,
						);
					}
				},
				onSuccess: () => {
					// Invalidate queries to ensure fresh data
					queryClient.invalidateQueries({ queryKey: inboxTasksQueryKey });
					queryClient.invalidateQueries({ queryKey: overdueTasksQueryKey });
					if (variables.goalId) {
						queryClient.invalidateQueries({
							queryKey: getGetGoalInstancesQueryKey(variables.goalId),
						});
					}
				},
			},
		);
	};

	return {
		...mutation,
		mutate,
	};
};

/**
 * Optimistic update subtask query helper
 */
const updateSubtaskInQueryCache = (
	queryClient: ReturnType<typeof useQueryClient>,
	taskId: string,
	subtaskId: string,
	updates: Partial<DbSubTask>,
) => {
	const subtasksQueryKey = getGetSubtasksQueryKey(taskId);

	// Update subtasks query directly
	queryClient.setQueryData<{ data: DbSubTask[] }>(subtasksQueryKey, (old) => {
		if (!old) return old;
		const updatedSubtasks = (old.data || []).map((subtask) =>
			subtask.id === subtaskId ? { ...subtask, ...updates } : subtask,
		);
		return { ...old, data: updatedSubtasks };
	});
};

/**
 * Optimistic update subtask
 */
export const useOptimisticUpdateSubtask = () => {
	const queryClient = useQueryClient();
	const mutation = useUpdateSubtask();

	const mutate = (
		variables: {
			taskId: string;
			subtaskId: string;
			data: { title?: string; isCompleted?: boolean };
		},
		options?: { onSuccess?: (data: unknown) => void; onError?: () => void },
	) => {
		const subtasksQueryKey = getGetSubtasksQueryKey(variables.taskId);

		// Store previous state for rollback
		const previousSubtasks = queryClient.getQueryData(subtasksQueryKey);

		// Optimistic update
		updateSubtaskInQueryCache(
			queryClient,
			variables.taskId,
			variables.subtaskId,
			variables.data,
		);

		// Perform actual mutation
		mutation.mutate(variables, {
			onSuccess: (data) => {
				options?.onSuccess?.(data);
			},
			onError: () => {
				// Rollback on error
				if (previousSubtasks) {
					queryClient.setQueryData(subtasksQueryKey, previousSubtasks);
				}
				options?.onError?.();
			},
		});
	};

	return {
		...mutation,
		mutate,
	};
};

/**
 * Wait for create subtask success, then update local cache
 */
export const useOptimisticCreateSubtask = () => {
	const queryClient = useQueryClient();
	const mutation = useCreateSubtask();

	const mutate = (
		variables: { taskId: string; data: { title: string } },
		options?: {
			onSuccess?: (data: { data: DbSubTask }) => void;
			onError?: () => void;
		},
	) => {
		const subtasksQueryKey = getGetSubtasksQueryKey(variables.taskId);
		const previousSubtasks = queryClient.getQueryData(subtasksQueryKey);

		mutation.mutate(variables, {
			onSuccess: (response) => {
				const newSubtask = response as { data: DbSubTask };
				queryClient.setQueryData<{ data: DbSubTask[] }>(
					subtasksQueryKey,
					(old) => {
						if (!old) return { data: [newSubtask.data] };
						return { ...old, data: [...(old.data || []), newSubtask.data] };
					},
				);
				options?.onSuccess?.(newSubtask);
			},
			onError: () => {
				// Rollback on error
				if (previousSubtasks) {
					queryClient.setQueryData(subtasksQueryKey, previousSubtasks);
				}
				options?.onError?.();
			},
		});
	};

	return {
		...mutation,
		mutate,
	};
};

/**
 * Optimistic delete subtask
 */
export const useOptimisticDeleteSubtask = () => {
	const queryClient = useQueryClient();
	const mutation = useDeleteSubtask();

	const mutate = (
		variables: { taskId: string; subtaskId: string },
		options?: {
			onSuccess?: () => void;
			onError?: () => void;
		},
	) => {
		const subtasksQueryKey = getGetSubtasksQueryKey(variables.taskId);
		const previousSubtasks = queryClient.getQueryData<{ data: DbSubTask[] }>(
			subtasksQueryKey,
		);

		// Optimistically update local cache
		queryClient.setQueryData<{ data: DbSubTask[] }>(subtasksQueryKey, (old) => {
			if (!old) return old;
			return {
				...old,
				data: old.data.filter((subtask) => subtask.id !== variables.subtaskId),
			};
		});

		mutation.mutate(variables, {
			onSuccess: () => {
				// If needed, further updates or success handlers go here
				options?.onSuccess?.();
			},
			onError: () => {
				// Rollback to previous subtasks list on error
				if (previousSubtasks) {
					queryClient.setQueryData(subtasksQueryKey, previousSubtasks);
				}
				options?.onError?.();
			},
		});
	};

	return {
		...mutation,
		mutate,
	};
};
