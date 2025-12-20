import { useQueryClient } from "@tanstack/react-query";
import {
	getGetInboxTasksQueryKey,
	getGetOverdueTasksQueryKey,
	useUpdateTask,
} from "~/aether-sdk";
import type { DbTask } from "~/aether-sdk/models/db-task";

export const useOptimisticUpdateTask = () => {
	const queryClient = useQueryClient();
	const mutation = useUpdateTask();

	const inboxTasksQueryKey = getGetInboxTasksQueryKey();

	const mutate = (variables: { id: string; data: Record<string, unknown> }) => {
		const previousTasks =
			queryClient.getQueryData<DbTask[]>(inboxTasksQueryKey);

		// Optimistically update
		queryClient.setQueryData<{ data: DbTask[] }>(inboxTasksQueryKey, (old) => {
			const oldData = old?.data || [];

			const updatedData = oldData.map((task) =>
				task.id === variables.id ? { ...task, ...variables.data } : task,
			);

			return {
				...old,
				data: updatedData,
			};

			// if (!oldData) return old;

			// return oldData.map((task) =>
			// 	task.id === variables.id ? { ...task, ...variables.data } : task,
			// );
		});

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
