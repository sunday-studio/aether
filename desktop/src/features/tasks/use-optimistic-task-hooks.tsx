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
import type { GoalInstance, PaginatedGoalInstances, PaginatedTasks, SubTask, Task } from "~/aether-sdk/models";

/**
 * Type for infinite query data structure
 */
type InfiniteQueryData<T> = {
	pages: Array<{ data: T }>;
	pageParams: unknown[];
};

/**
 * Helper to update a task in infinite query cache
 */
const updateTaskInInfiniteCache = <T extends { items?: Task[] }>(
	queryClient: ReturnType<typeof useQueryClient>,
	queryKey: unknown[],
	taskId: string,
	updates: Record<string, unknown>,
) => {
	queryClient.setQueryData<InfiniteQueryData<T>>(queryKey, (old) => {
		if (!old?.pages) return old;
		return {
			...old,
			pages: old.pages.map((page) => ({
				...page,
				data: {
					...page.data,
					items: (page.data?.items || []).map((task) =>
						task.id === taskId ? { ...task, ...updates } : task,
					),
				},
			})),
		};
	});
};

/**
 * Helper to remove a task from infinite query cache
 */
const removeTaskFromInfiniteCache = <T extends { items?: Task[] }>(
	queryClient: ReturnType<typeof useQueryClient>,
	queryKey: unknown[],
	taskId: string,
) => {
	queryClient.setQueryData<InfiniteQueryData<T>>(queryKey, (old) => {
		if (!old?.pages) return old;
		return {
			...old,
			pages: old.pages.map((page) => ({
				...page,
				data: {
					...page.data,
					items: (page.data?.items || []).filter((task) => task.id !== taskId),
				},
			})),
		};
	});
};

/**
 * Optimistic update task query
 */
export const useOptimisticUpdateTaskQuery = () => {
	const queryClient = useQueryClient();
	const inboxTasksQueryKey = getGetInboxTasksQueryKey();
	const overdueTasksQueryKey = getGetOverdueTasksQueryKey();

	const getPreviousData = () => ({
		inbox: queryClient.getQueryData<InfiniteQueryData<PaginatedTasks>>(inboxTasksQueryKey),
		overdue: queryClient.getQueryData<InfiniteQueryData<PaginatedTasks>>(overdueTasksQueryKey),
	});

	const updateLocalInstance = (variables: {
		id: string;
		data: Record<string, unknown>;
	}) => {
		// Update inbox tasks
		updateTaskInInfiniteCache<PaginatedTasks>(
			queryClient,
			inboxTasksQueryKey,
			variables.id,
			variables.data,
		);

		// Update overdue tasks
		updateTaskInInfiniteCache<PaginatedTasks>(
			queryClient,
			overdueTasksQueryKey,
			variables.id,
			variables.data,
		);
	};

	return {
		updateLocalInstance,
		getPreviousData,
		queryClient,
		inboxTasksQueryKey,
		overdueTasksQueryKey,
	};
};

export const useOptimisticUpdateTask = () => {
	const { updateLocalInstance, getPreviousData, queryClient, inboxTasksQueryKey, overdueTasksQueryKey } = useOptimisticUpdateTaskQuery();
	const mutation = useUpdateTask();

	const mutate = (variables: { id: string; data: Record<string, unknown>; goalId?: string }) => {
		// Store previous state for rollback
		const previousData = getPreviousData();
		const previousGoalInstances = variables.goalId
			? queryClient.getQueryData(getGetGoalInstancesQueryKey(variables.goalId))
			: null;

		// Optimistically update inbox and overdue
		updateLocalInstance(variables);

		// Optimistically update goal instances if goalId is provided
		if (variables.goalId) {
			const goalInstancesQueryKey = getGetGoalInstancesQueryKey(variables.goalId);
			queryClient.setQueryData<InfiniteQueryData<PaginatedGoalInstances>>(
				goalInstancesQueryKey,
				(old) => {
					if (!old?.pages) return old;
					return {
						...old,
						pages: old.pages.map((page) => ({
							...page,
							data: {
								...page.data,
								items: (page.data?.items || []).map((goalInstance) => ({
									...goalInstance,
									tasks: ((goalInstance as GoalInstance & { tasks?: Task[] }).tasks || []).map(
										(task) => task.id === variables.id ? { ...task, ...variables.data } : task,
									),
								})),
							},
						})),
					};
				},
			);
		}

		mutation.mutate(variables, {
			onError: () => {
				// Rollback all caches
				if (previousData.inbox) {
					queryClient.setQueryData(inboxTasksQueryKey, previousData.inbox);
				}
				if (previousData.overdue) {
					queryClient.setQueryData(overdueTasksQueryKey, previousData.overdue);
				}
				if (previousGoalInstances && variables.goalId) {
					queryClient.setQueryData(getGetGoalInstancesQueryKey(variables.goalId), previousGoalInstances);
				}
			},
		});
	};

	return {
		...mutation,
		mutate,
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
		removeTaskFromInfiniteCache<PaginatedTasks>(queryClient, inboxTasksQueryKey, variables.id);

		// Optimistically update overdue tasks
		removeTaskFromInfiniteCache<PaginatedTasks>(queryClient, overdueTasksQueryKey, variables.id);

		// Optimistically update goal instances if goalId is provided
		if (variables.goalId) {
			const goalInstancesQueryKey = getGetGoalInstancesQueryKey(variables.goalId);
			queryClient.setQueryData<InfiniteQueryData<PaginatedGoalInstances>>(
				goalInstancesQueryKey,
				(old) => {
					if (!old?.pages) return old;
					return {
						...old,
						pages: old.pages.map((page) => ({
							...page,
							data: {
								...page.data,
								items: (page.data?.items || []).map((goalInstance) => ({
									...goalInstance,
									tasks: ((goalInstance as GoalInstance & { tasks?: Task[] }).tasks || []).filter(
										(task) => task.id !== variables.id,
									),
								})),
							},
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
						queryClient.setQueryData(overdueTasksQueryKey, previousOverdueTasks);
					}
					if (previousGoalInstances && variables.goalId) {
						queryClient.setQueryData(
							getGetGoalInstancesQueryKey(variables.goalId),
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
	updates: Partial<SubTask>,
) => {
	const subtasksQueryKey = getGetSubtasksQueryKey(taskId);

	// Update subtasks query directly
	queryClient.setQueryData<{ data: SubTask[] }>(subtasksQueryKey, (old) => {
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
 * Optimistic create subtask - adds temporary subtask immediately
 */
export const useOptimisticCreateSubtask = () => {
	const queryClient = useQueryClient();
	const mutation = useCreateSubtask();

	const mutate = (
		variables: { taskId: string; data: { title: string } },
		options?: {
			onSuccess?: (data: { data: SubTask }) => void;
			onError?: () => void;
		},
	) => {
		const subtasksQueryKey = getGetSubtasksQueryKey(variables.taskId);
		const previousSubtasks = queryClient.getQueryData(subtasksQueryKey);

		// Create a temporary subtask with optimistic ID
		const tempId = `temp-${Date.now()}`;
		const optimisticSubtask: SubTask = {
			id: tempId,
			title: variables.data.title,
			isCompleted: false,
			taskId: variables.taskId,
			createdAt: new Date().toISOString(),
			updatedAt: new Date().toISOString(),
		};

		// Optimistically add the subtask
		queryClient.setQueryData<{ data: SubTask[] }>(subtasksQueryKey, (old) => {
			if (!old) return { data: [optimisticSubtask] };
			return { ...old, data: [...(old.data || []), optimisticSubtask] };
		});

		mutation.mutate(variables, {
			onSuccess: (response) => {
				const newSubtask = response as { data: SubTask };
				// Replace the optimistic subtask with the real one
				queryClient.setQueryData<{ data: SubTask[] }>(
					subtasksQueryKey,
					(old) => {
						if (!old) return { data: [newSubtask.data] };
						return {
							...old,
							data: old.data.map((subtask) =>
								subtask.id === tempId ? newSubtask.data : subtask,
							),
						};
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
		const previousSubtasks = queryClient.getQueryData<{ data: SubTask[] }>(
			subtasksQueryKey,
		);

		// Optimistically update local cache
		queryClient.setQueryData<{ data: SubTask[] }>(subtasksQueryKey, (old) => {
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
