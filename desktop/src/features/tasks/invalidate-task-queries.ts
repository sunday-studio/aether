import type { QueryClient } from '@tanstack/react-query';
import {
	getGetGoalInstancesInfiniteQueryKey,
	getGetInboxTasksInfiniteQueryKey,
	getGetOverdueTasksInfiniteQueryKey,
	getGetSubtasksQueryKey,
} from '~/aether-sdk';
import { invalidateSearchQueries } from '~/lib/search-query-invalidation';

/**
 * Invalidate inbox + overdue + optionally goal instances for a goal.
 * Call after create/update/delete task (pass goalId when task is scoped to a goal).
 * Also refetches so the list updates immediately.
 */
export function invalidateTaskQueries(queryClient: QueryClient, options?: { goalId?: string }) {
	const inboxKey = getGetInboxTasksInfiniteQueryKey({});
	const overdueKey = getGetOverdueTasksInfiniteQueryKey({});
	queryClient.invalidateQueries({ queryKey: inboxKey });
	queryClient.invalidateQueries({ queryKey: overdueKey });
	invalidateSearchQueries(queryClient);
	void queryClient.refetchQueries({ queryKey: inboxKey });
	void queryClient.refetchQueries({ queryKey: overdueKey });
	if (options?.goalId) {
		const goalKey = getGetGoalInstancesInfiniteQueryKey(options.goalId, {});
		queryClient.invalidateQueries({ queryKey: goalKey });
		void queryClient.refetchQueries({ queryKey: goalKey });
	}
}

/**
 * Invalidate subtasks query for a task.
 * Call after create/update/delete subtask.
 * When subtasks are shown from parent task list data, also call invalidateTaskQueries
 * so inbox/goal instances refetch and task.subtasks updates.
 */
export function invalidateSubtaskQueries(queryClient: QueryClient, taskId: string) {
	queryClient.invalidateQueries({
		queryKey: getGetSubtasksQueryKey(taskId),
	});
}
