import type { DbTask } from "~/aether-sdk/models";

export const groupTaskByCreatedAt = (tasks: DbTask[]) => {
	const groupedTasks: Record<string, DbTask[]> = {};

	tasks.forEach((task) => {
		const dateKey = task.createdAt
			? new Date(task.createdAt ?? "").toISOString().split("T")[0]
			: "unknown";
		if (!groupedTasks[dateKey]) {
			groupedTasks[dateKey] = [];
		}
		groupedTasks[dateKey].push(task);
	});

	// Sort the grouped keys
	const sortedGroupedTasks: Record<string, DbTask[]> = {};
	Object.keys(groupedTasks)
		.sort((a, b) => new Date(b).getTime() - new Date(a).getTime())
		.forEach((key) => {
			sortedGroupedTasks[key] = groupedTasks[key];
		});

	return sortedGroupedTasks;
};
