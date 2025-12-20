import { useQueryClient } from "@tanstack/react-query";
import { getGetInboxTasksQueryKey, useUpdateTask } from "~/aether-sdk";
import type { DbTask } from "~/aether-sdk/models/db-task";

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

export const useOptimisticUpdateTaskQuery = () => {
	const queryClient = useQueryClient();
	const inboxTasksQueryKey = getGetInboxTasksQueryKey();
	const previousTasks = queryClient.getQueryData<DbTask[]>(inboxTasksQueryKey);

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
