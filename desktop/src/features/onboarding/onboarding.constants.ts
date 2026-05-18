import { Server, Sparkles, UserRound, WandSparkles, type LucideIcon } from 'lucide-react';
import { type ProviderChoice, type StepId } from './onboarding.types';

export const ONBOARDING_COMPLETED_KEY = 'app.onboarding_completed';
export const DISPLAY_NAME_KEY = 'user.display_name';
export const RECOVERY_SEED_KEY = 'user.recovery_seed_phrase';
export const DEFAULT_PROVIDER_KEY = 'transcription.default_provider';
export const OPENAI_API_KEY = 'transcription.openai.api_key';
export const GROQ_API_KEY = 'transcription.groq.api_key';
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
	'canvas',
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

export const providerCopy: Record<
	ProviderChoice,
	{ label: string; description: string; keyLabel: string; placeholder: string }
> = {
	openai: {
		label: 'OpenAI Whisper',
		description: 'Reliable hosted transcription if you already use OpenAI.',
		keyLabel: 'OpenAI API Key',
		placeholder: 'sk-...',
	},
	groq: {
		label: 'Groq',
		description: "Fast hosted transcription with Groq's API.",
		keyLabel: 'Groq API Key',
		placeholder: 'gsk_...',
	},
};

export const steps: Array<{ id: StepId; label: string; icon: LucideIcon }> = [
	{ id: 'intro', label: 'Welcome', icon: Sparkles },
	{ id: 'profile', label: 'Identity', icon: UserRound },
	{ id: 'sync', label: 'Sync', icon: Server },
	{ id: 'ai', label: 'AI', icon: WandSparkles },
];
