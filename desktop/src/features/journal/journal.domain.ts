import { eachDayOfInterval, format, subMonths } from "date-fns";
import type { EntryWithTags } from "~/types/models";

/**
 * Recursively extract text from a Lexical node
 */
const extractTextFromNode = (node: any, text: string[]): void => {
	if (!node) return;

	if (typeof node === "object") {
		// Extract text from text nodes
		if (node.type === "text" && node.text) {
			text.push(node.text);
		}

		// Recursively process children
		if (node.children && Array.isArray(node.children)) {
			for (const child of node.children) {
				extractTextFromNode(child, text);
			}
		}
	}
};

/**
 * Extract text content from a Lexical document JSON string
 */
export const extractTextFromLexical = (document: string): string => {
	try {
		const doc = JSON.parse(document || "{}");
		const textParts: string[] = [];
		extractTextFromNode(doc.root, textParts);
		return textParts.join("");
	} catch {
		return "";
	}
};

/**
 * Extract the first sentence from a Lexical document
 * Returns the first sentence (ending with . ! or ?) or first line if no sentence ending found
 */
export const extractFirstSentence = (document: string): string => {
	const text = extractTextFromLexical(document);
	if (!text.trim()) return "Untitled Entry";

	// Try to find first sentence ending
	const sentenceMatch = text.match(/^[^.!?]*[.!?]/);
	if (sentenceMatch) {
		return sentenceMatch[0].trim();
	}

	// Fallback to first line or first 100 characters
	const firstLine = text.split("\n")[0].trim();
	if (firstLine) {
		return firstLine.length > 100
			? firstLine.substring(0, 100) + "..."
			: firstLine;
	}

	return text.substring(0, 100) || "Untitled Entry";
};

/**
 * Group entries by tags
 * Returns a map where keys are tag IDs and values are arrays of entries
 * Entries without tags are grouped under "untagged"
 * Entries are sorted within each group by creation date (newest first)
 */
export const groupEntriesByTags = (
	entries: EntryWithTags[],
): Map<string, { tagName: string; entries: EntryWithTags[] }> => {
	const grouped = new Map<
		string,
		{ tagName: string; entries: EntryWithTags[] }
	>();

	// Add untagged group
	const untaggedGroup = { tagName: "Untagged", entries: [] as EntryWithTags[] };

	for (const entry of entries) {
		if (!entry.tags || entry.tags.length === 0) {
			untaggedGroup.entries.push(entry);
		} else {
			// Add entry to each of its tags
			for (const tag of entry.tags) {
				const tagId = tag.id ?? "";
				const tagName = tag.name ?? "Unknown Tag";

				if (!grouped.has(tagId)) {
					grouped.set(tagId, { tagName, entries: [] });
				}
				grouped.get(tagId)?.entries.push(entry);
			}
		}
	}

	// Sort entries within each group and only add untagged group if it has entries
	if (untaggedGroup.entries.length > 0) {
		untaggedGroup.entries = sortEntries(untaggedGroup.entries);
		grouped.set("untagged", untaggedGroup);
	}

	// Sort entries within each tag group
	for (const [tagId, group] of grouped.entries()) {
		if (tagId !== "untagged") {
			group.entries = sortEntries(group.entries);
		}
	}

	return grouped;
};

export const sortEntries = (entries: EntryWithTags[]) => {
	return entries?.sort((a, b) => {
		return (
			new Date(b.createdAt ?? "").getTime() -
			new Date(a.createdAt ?? "").getTime()
		);
	});
};

export const normalizeEntries = (entries: EntryWithTags[]) => {
	const groupedEntries: Record<string, EntryWithTags[]> = {};

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

export const normalizeEntriesToMap = (entries: EntryWithTags[]) => {
	const map = new Map<string, EntryWithTags[]>();
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
