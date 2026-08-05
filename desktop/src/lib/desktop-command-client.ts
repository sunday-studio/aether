import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { createCommandClient } from './command-client';

/** Desktop shell implementation of the shared command-client contract. */
export const desktopCommandClient = createCommandClient((command, args) =>
	tauriInvoke(command, args),
);
