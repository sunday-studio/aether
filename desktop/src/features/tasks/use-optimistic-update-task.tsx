import { useQueryClient } from "@tanstack/react-query";
import {
	getGetInboxTasksQueryKey,
	useDeleteTaskById,
	useUpdateTask,
} from "~/aether-sdk";
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
	const previousTasks = queryClient.getQueryData<{ data: DbTask[] }>(
		inboxTasksQueryKey,
	);
	const mutation = useDeleteTaskById();

	const mutate = (variables: { id: string }) => {
		// Optimistically update cache: remove the task from the cache
		queryClient.setQueryData<{ data: DbTask[] }>(inboxTasksQueryKey, (old) => {
			const oldData = old?.data || [];
			return {
				...old,
				data: oldData.filter((task) => task.id !== variables.id),
			};
		});

		mutation.mutate(variables, {
			onError: () => {
				// Rollback to previous cache state if error
				queryClient.setQueryData(inboxTasksQueryKey, previousTasks);
			},
		});
	};

	return {
		...mutation,
		mutate,
		previousTasks,
	};
};
