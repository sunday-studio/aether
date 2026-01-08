import { useQueryClient } from "@tanstack/react-query";
import {
	getGetInboxTasksQueryKey,
	getGetOverdueTasksQueryKey,
	getGetGoalInstancesQueryKey,
	useDeleteTaskById,
	useUpdateTask,
} from "~/aether-sdk";
import type { DbGoalInstance } from "~/aether-sdk/models/db-goal-instance";
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
			? queryClient.getQueryData(
					getGetGoalInstancesQueryKey(variables.goalId),
				)
			: null;

		// Optimistically update inbox tasks
		queryClient.setQueryData<{ data: DbTask[] }>(
			inboxTasksQueryKey,
			(old) => {
				if (!old) return old;
				const oldData = old.data || [];
				return {
					...old,
					data: oldData.filter((task) => task.id !== variables.id),
				};
			},
		);

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
