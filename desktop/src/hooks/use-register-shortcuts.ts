import { useHotkeys } from "react-hotkeys-hook";
import { useNavigate } from "react-router";
import { useCreateJournalEntry } from "./use-create-journal-entry";

enum Key {
	Meta = "Meta",
	Control = "Control",
	Shift = "Shift",
	Alt = "Alt",
	Tab = "Tab",
	Enter = "Enter",
	Escape = "Escape",
}

const shortcuts = {
	CREATE_NEW_ENTRY: `${Key.Meta}+n`,

	// Routing
	NAVIGATE_TO_JOURNAL: `${Key.Meta}+j`,
	NAVIGATE_TO_TASKS: `${Key.Meta}+t`,
	NAVIGATE_TO_CANVAS: `${Key.Meta}+c`,
	NAVIGATE_TO_SETTINGS: `${Key.Meta}+s`,
};

export const useRegisterShortcuts = () => {
	const navigate = useNavigate();
	const { createEntry } = useCreateJournalEntry();

	useHotkeys(shortcuts.CREATE_NEW_ENTRY, createEntry);
	useHotkeys(shortcuts.NAVIGATE_TO_JOURNAL, () => navigate("/"));
	useHotkeys(shortcuts.NAVIGATE_TO_TASKS, () => navigate("/tasks"));
	useHotkeys(shortcuts.NAVIGATE_TO_CANVAS, () => navigate("/canvas"));
	useHotkeys(shortcuts.NAVIGATE_TO_SETTINGS, () => navigate("/settings"));
};
