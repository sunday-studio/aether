import * as React from 'react';
import { useHotkeys } from 'react-hotkeys-hook';
import { useNavigate } from 'react-router';
import { useCreateJournalEntry } from './use-create-journal-entry';

enum Key {
	Meta = 'Meta',
	Control = 'Control',
	Shift = 'Shift',
	Alt = 'Alt',
	Tab = 'Tab',
	Enter = 'Enter',
	Escape = 'Escape',
}

const shortcuts = {
	CREATE_NEW_ENTRY: `${Key.Meta}+n`,
	OPEN_COMMAND_PALETTE: `${Key.Meta}+k`,

	// Routing
	NAVIGATE_TO_JOURNAL: `${Key.Meta}+j`,
	NAVIGATE_TO_TASKS: `${Key.Meta}+t`,
	NAVIGATE_TO_SETTINGS: `${Key.Meta}+s`,
};

export const useRegisterShortcuts = () => {
	const navigate = useNavigate();
	const { createEntry } = useCreateJournalEntry();
	const [commandPaletteOpen, setCommandPaletteOpen] = React.useState(false);

	useHotkeys(shortcuts.CREATE_NEW_ENTRY, createEntry);
	useHotkeys(shortcuts.OPEN_COMMAND_PALETTE, () => setCommandPaletteOpen(true));
	useHotkeys(shortcuts.NAVIGATE_TO_JOURNAL, () => navigate('/'));
	useHotkeys(shortcuts.NAVIGATE_TO_TASKS, () => navigate('/tasks'));
	useHotkeys(shortcuts.NAVIGATE_TO_SETTINGS, () => navigate('/settings'));

	return { commandPaletteOpen, setCommandPaletteOpen };
};
