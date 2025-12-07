import { eachDayOfInterval, format, subMonths } from "date-fns";
import type { DbEntry } from "~/aether-sdk/models";

export const sortEntries = (entries: DbEntry[]) => {
	return entries.sort((a, b) => {
		return (
			new Date(b.createdAt ?? "").getTime() -
			new Date(a.createdAt ?? "").getTime()
		);
	});
};

export const normalizeEntries = (entries: DbEntry[]) => {
	const groupedEntries: Record<string, DbEntry[]> = {};

	entries.forEach((entry) => {
		const dateKey = entry.createdAt
			? new Date(entry.createdAt).toISOString().split("T")[0]
			: "unknown";
		if (!groupedEntries[dateKey]) {
			groupedEntries[dateKey] = [];
		}
		groupedEntries[dateKey].push(entry);
	});

	return groupedEntries;
};

/**
 * Generates an array of days from the start date to the end date
 * @param startDate - The start date
 * @returns An array of days
 */
export const generateDays = (
	startDate: Date = new Date(),
	months: number = 6,
): Date[] => {
	const end = startDate;
	const start = subMonths(end, months);

	return eachDayOfInterval({ start, end }).reverse();
};

export const getDateKey = (date: Date) => {
	return format(date, "yyyy-MM-dd");
};

export const normalizeEntriesToMap = (entries: DbEntry[]) => {
	const map = new Map<string, DbEntry[]>();
	for (const entry of entries) {
		const dateKey = entry.createdAt
			? new Date(entry.createdAt).toISOString()
			: "unknown";
		if (!map.has(dateKey)) {
			map.set(dateKey, []);
		}
		map.get(dateKey)?.push(entry);
	}
	return map;
};
