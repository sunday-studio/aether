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

const hotkeyOptions = {
	preventDefault: true,
};

export const useRegisterShortcuts = () => {
	const navigate = useNavigate();
	const { createEntry } = useCreateJournalEntry();
	const [commandPaletteOpen, setCommandPaletteOpen] = React.useState(false);

	useHotkeys(shortcuts.CREATE_NEW_ENTRY, createEntry, hotkeyOptions);
	useHotkeys(shortcuts.OPEN_COMMAND_PALETTE, () => setCommandPaletteOpen(true), hotkeyOptions);
	useHotkeys(shortcuts.NAVIGATE_TO_JOURNAL, () => navigate('/'), hotkeyOptions);
	useHotkeys(shortcuts.NAVIGATE_TO_TASKS, () => navigate('/tasks'), hotkeyOptions);
	useHotkeys(shortcuts.NAVIGATE_TO_SETTINGS, () => navigate('/settings'), hotkeyOptions);

	return { commandPaletteOpen, setCommandPaletteOpen };
};
