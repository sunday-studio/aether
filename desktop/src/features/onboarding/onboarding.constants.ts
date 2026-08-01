import { HardDrive, Server, Sparkles, UserRound, type LucideIcon } from 'lucide-react';
import { type StepId } from './onboarding.types';

export const ONBOARDING_COMPLETED_KEY = 'app.onboarding_completed';
export const DISPLAY_NAME_KEY = 'user.display_name';
export const RECOVERY_SEED_KEY = 'user.recovery_seed_phrase';
export const SEARCH_EMBEDDINGS_ENABLED_KEY = 'search.embeddings.enabled';
export const SEARCH_EMBEDDINGS_PROVIDER_KEY = 'search.embeddings.provider';
export const SEARCH_EMBEDDINGS_MODEL_KEY = 'search.embeddings.model';
export const SEARCH_EMBEDDINGS_AUTO_INDEX_KEY = 'search.embeddings.auto_index';
export const SYNC_GUIDE_URL =
	'https://github.com/sunday-studio/aether/blob/main/docs/reference/sync-server-readme.md';

export const recoveryWords = [
	'aether',
	'anchor',
	'archive',
	'atelier',
	'bloom',
	'cipher',
	'compass',
	'ember',
	'field',
	'glimmer',
	'harbor',
	'journal',
	'kernel',
	'lantern',
	'ledger',
	'meadow',
	'notebook',
	'orbit',
	'parcel',
	'quartz',
	'river',
	'signal',
	'studio',
	'thread',
	'vault',
	'velvet',
	'window',
];

export const steps: Array<{ id: StepId; label: string; icon: LucideIcon }> = [
	{ id: 'intro', label: 'Welcome', icon: Sparkles },
	{ id: 'profile', label: 'Identity', icon: UserRound },
	{ id: 'sync', label: 'Sync', icon: Server },
	{ id: 'search', label: 'Search', icon: HardDrive },
];
