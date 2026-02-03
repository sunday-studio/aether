import type { QueryClient } from "@tanstack/react-query";
import {
	getGetGoalInstancesInfiniteQueryKey,
	getGetInboxTasksInfiniteQueryKey,
	getGetOverdueTasksInfiniteQueryKey,
	getGetSubtasksQueryKey,
} from "~/aether-sdk";

/**
 * Invalidate inbox + overdue + optionally goal instances for a goal.
 * Call after create/update/delete task (pass goalId when task is scoped to a goal).
 */
export function invalidateTaskQueries(
	queryClient: QueryClient,
	options?: { goalId?: string },
) {
	queryClient.invalidateQueries({ queryKey: getGetInboxTasksInfiniteQueryKey({}) });
	queryClient.invalidateQueries({
		queryKey: getGetOverdueTasksInfiniteQueryKey({}),
	});
	if (options?.goalId) {
		queryClient.invalidateQueries({
			queryKey: getGetGoalInstancesInfiniteQueryKey(options.goalId, {}),
		});
	}
}

/**
 * Invalidate subtasks query for a task.
 * Call after create/update/delete subtask.
 */
export function invalidateSubtaskQueries(
	queryClient: QueryClient,
	taskId: string,
) {
	queryClient.invalidateQueries({
		queryKey: getGetSubtasksQueryKey(taskId),
	});
}
