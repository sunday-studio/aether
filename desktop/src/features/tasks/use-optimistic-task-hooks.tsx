import { useQueryClient } from "@tanstack/react-query";
import {
	getGetGoalInstancesQueryKey,
	getGetInboxTasksQueryKey,
	getGetOverdueTasksQueryKey,
	getGetSubTasksQueryKey,
	useDeleteTaskById,
	useUpdateSubTask,
	useUpdateTask,
} from "~/aether-sdk";
import type { DbGoalInstance } from "~/aether-sdk/models/db-goal-instance";
import type { DbSubTask } from "~/aether-sdk/models/db-sub-task";
import type { DbTask } from "~/aether-sdk/models/db-task";

/**
 * Optimistic update task
 */
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

	const mutation = useDeleteTaskById();

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
	const subtasksQueryKey = getGetSubTasksQueryKey(taskId);
	const inboxTasksQueryKey = getGetInboxTasksQueryKey();
	const overdueTasksQueryKey = getGetOverdueTasksQueryKey();

	// Update subtasks query directly
	queryClient.setQueryData<{ data: DbSubTask[] }>(subtasksQueryKey, (old) => {
		if (!old) return old;
		const updatedSubtasks = (old.data || []).map((subtask) =>
			subtask.id === subtaskId ? { ...subtask, ...updates } : subtask,
		);
		return { ...old, data: updatedSubtasks };
	});

	const updateTaskSubtasks = (tasks: DbTask[] | undefined): DbTask[] => {
		if (!tasks) return [];
		return tasks.map((task) => {
			if (task.id === taskId) {
				const updatedSubtasks = (task.subTasks || []).map((subtask) =>
					subtask.id === subtaskId ? { ...subtask, ...updates } : subtask,
				);
				return { ...task, subTasks: updatedSubtasks };
			}
			return task;
		});
	};

	// Update inbox tasks
	queryClient.setQueryData<{ data: DbTask[] }>(inboxTasksQueryKey, (old) => {
		if (!old) return old;
		return { ...old, data: updateTaskSubtasks(old.data) };
	});

	// Update overdue tasks
	queryClient.setQueryData<{ data: DbTask[] }>(overdueTasksQueryKey, (old) => {
		if (!old) return old;
		return { ...old, data: updateTaskSubtasks(old.data) };
	});

	// Update goal instances (if task has a goalId)
	const inboxTasks = queryClient.getQueryData<{ data: DbTask[] }>(
		inboxTasksQueryKey,
	);
	const task = inboxTasks?.data?.find((t) => t.id === taskId);
	if (task?.goalId) {
		const goalInstancesQueryKey = getGetGoalInstancesQueryKey(task.goalId);
		queryClient.setQueryData<{ data: DbGoalInstance[] }>(
			goalInstancesQueryKey,
			(old) => {
				if (!old) return old;
				return {
					...old,
					data: old.data.map((goalInstance) => ({
						...goalInstance,
						tasks: updateTaskSubtasks(goalInstance.tasks),
					})),
				};
			},
		);
	}
};

/**
 * Optimistic update subtask
 */
export const useOptimisticUpdateSubtask = () => {
	const queryClient = useQueryClient();
	const mutation = useUpdateSubTask();

	const inboxTasksQueryKey = getGetInboxTasksQueryKey();
	const overdueTasksQueryKey = getGetOverdueTasksQueryKey();

	const mutate = (variables: {
		taskId: string;
		subtaskId: string;
		data: { title?: string; isCompleted?: boolean };
	}) => {
		const subtasksQueryKey = getGetSubTasksQueryKey(variables.taskId);

		// Store previous state for rollback
		const previousSubtasks = queryClient.getQueryData(subtasksQueryKey);
		const previousInboxTasks = queryClient.getQueryData(inboxTasksQueryKey);
		const previousOverdueTasks = queryClient.getQueryData(overdueTasksQueryKey);

		// Get task to check for goalId
		const inboxTasks = queryClient.getQueryData<{ data: DbTask[] }>(
			inboxTasksQueryKey,
		);
		const task = inboxTasks?.data?.find((t) => t.id === variables.taskId);
		const previousGoalInstances = task?.goalId
			? queryClient.getQueryData(getGetGoalInstancesQueryKey(task.goalId))
			: null;

		// Optimistic update
		updateSubtaskInQueryCache(
			queryClient,
			variables.taskId,
			variables.subtaskId,
			variables.data,
		);

		// Perform actual mutation
		mutation.mutate(variables, {
			onError: () => {
				// Rollback on error
				if (previousSubtasks) {
					queryClient.setQueryData(subtasksQueryKey, previousSubtasks);
				}
				if (previousInboxTasks) {
					queryClient.setQueryData(inboxTasksQueryKey, previousInboxTasks);
				}
				if (previousOverdueTasks) {
					queryClient.setQueryData(overdueTasksQueryKey, previousOverdueTasks);
				}
				if (previousGoalInstances && task?.goalId) {
					queryClient.setQueryData(
						getGetGoalInstancesQueryKey(task.goalId),
						previousGoalInstances,
					);
				}
			},
		});
	};

	return {
		...mutation,
		mutate,
	};
};
