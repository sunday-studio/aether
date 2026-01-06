import type { DbGoal, DbGoalInstance, DbTask } from "~/aether-sdk/models";

export enum RecurrenceType {
	WEEKLY = "weekly",
	BI_WEEKLY = "bi-weekly",
	MONTHLY = "monthly",
	CUSTOM = "custom",
}

export const groupTaskByCreatedAt = (tasks: DbTask[]) => {
	if (tasks.length === 0) return {};

	const groupedTasks: Record<string, DbTask[]> = {};

	tasks?.forEach((task) => {
		const dateKey = task?.createdAt
			? new Date(task?.createdAt ?? "").toISOString().split("T")[0]
			: "unknown";
		if (!groupedTasks[dateKey]) {
			groupedTasks[dateKey] = [];
		}
		groupedTasks[dateKey].push(task);
	});

	// Sort the grouped keys and the arrays of tasks in each key
	const sortedGroupedTasks: Record<string, DbTask[]> = {};
	Object.keys(groupedTasks)
		.sort((a, b) => new Date(b).getTime() - new Date(a).getTime())
		.forEach((key) => {
			// Sort the tasks for each dateKey by createdAt DESC
			sortedGroupedTasks[key] = groupedTasks[key].slice().sort((a, b) => {
				const aTime = a.createdAt ? new Date(a.createdAt).getTime() : 0;
				const bTime = b.createdAt ? new Date(b.createdAt).getTime() : 0;
				return bTime - aTime;
			});
		});

	return sortedGroupedTasks;
};

export const transformGoalInstancesToGroupedTasks = (
	goalInstances: DbGoalInstance[],
) => {
	const groupedTasks: Record<string, DbTask[]> = {};

	goalInstances.forEach((goalInstance) => {
		const dateKey = goalInstance.periodStart
			? new Date(goalInstance.periodStart).toISOString().split("T")[0]
			: "unknown";
		if (!groupedTasks[dateKey]) {
			groupedTasks[dateKey] = [];
		}
		groupedTasks[dateKey].push(...(goalInstance.tasks ?? []));
	});

	return groupedTasks;
};

// TODO: relook into this
export const generateGoalInstanceTitle = (
	goalInstance: DbGoalInstance | null,
	goal: DbGoal | null,
) => {
	if (!goalInstance || !goal) return "";

	switch (goal.recurrenceType) {
		case RecurrenceType.WEEKLY:
			return `Week ${goalInstance.periodStart}`;
		case RecurrenceType.BI_WEEKLY:
			return `Bi-Week ${goalInstance.periodStart}`;
		case RecurrenceType.MONTHLY:
			return `Month ${goalInstance.periodStart}`;
	}
};
