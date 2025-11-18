import { DbEntry } from "~/aether-sdk/models";

export const normalizeEntries = (entries: DbEntry[]) => {
	const groupedEntries: Record<string, DbEntry[]> = {};
	
	entries.forEach((entry) => {
		const dateKey = entry.createdAt ? new Date(entry.createdAt).toISOString().split("T")[0] : "unknown";
		if (!groupedEntries[dateKey]) {
			groupedEntries[dateKey] = [];
		}
		groupedEntries[dateKey].push(entry);
	});

	return groupedEntries;
};