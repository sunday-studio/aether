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
