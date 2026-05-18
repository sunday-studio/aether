import { ExternalLink } from 'lucide-react';
import { motion } from 'motion/react';
import { TextField } from '~/components/shared/text-field';
import { SYNC_GUIDE_URL } from '../onboarding.constants';
import { type SyncChoice } from '../onboarding.types';
import { ChoiceButton } from './choice-button';

interface SyncStepProps {
	syncChoice: SyncChoice;
	serverUrl: string;
	serverSeedPhrase: string;
	syncPassphrase: string;
	onSyncChoiceChange: (value: SyncChoice) => void;
	onServerUrlChange: (value: string) => void;
	onServerSeedPhraseChange: (value: string) => void;
	onSyncPassphraseChange: (value: string) => void;
}

export function SyncStep({
	syncChoice,
	serverUrl,
	serverSeedPhrase,
	syncPassphrase,
	onSyncChoiceChange,
	onServerUrlChange,
	onServerSeedPhraseChange,
	onSyncPassphraseChange,
}: SyncStepProps) {
	return (
		<div className='space-y-6'>
			<div>
				<h3 className='text-3xl font-medium'>Do you already have a sync server?</h3>
				<p className='mt-3 max-w-xl text-sm leading-6 text-(--color-secondary-text)'>
					Aether sync is end-to-end encrypted and self-hosted. Connect an existing server now, or
					use the setup guide later. The server seed phrase enrolls this device; the sync passphrase
					protects your data before it reaches the server.
				</p>
			</div>
			<div className='grid gap-3 md:grid-cols-2'>
				<ChoiceButton
					isSelected={syncChoice === 'yes'}
					onClick={() => onSyncChoiceChange('yes')}
					title='Yes, connect it'
					copy='I have a server URL, server seed phrase, and sync passphrase.'
				/>
				<ChoiceButton
					isSelected={syncChoice === 'no'}
					onClick={() => onSyncChoiceChange('no')}
					title='No, show me how'
					copy='I will set up self-hosted sync later.'
				/>
			</div>
			{syncChoice === 'yes' && (
				<motion.div
					className='grid gap-3'
					initial={{ opacity: 0, height: 0 }}
					animate={{ opacity: 1, height: 'auto' }}
				>
					<TextField
						label='Server URL'
						placeholder='https://your-sync-server:8080'
						value={serverUrl}
						onChange={onServerUrlChange}
					/>
					<div className='grid gap-3 md:grid-cols-2'>
						<TextField
							label='Server seed phrase'
							placeholder='min 12 characters'
							type='password'
							value={serverSeedPhrase}
							onChange={onServerSeedPhraseChange}
						/>
						<TextField
							label='Sync passphrase'
							placeholder='min 12 characters'
							type='password'
							value={syncPassphrase}
							onChange={onSyncPassphraseChange}
						/>
					</div>
					<div className='grid gap-3 text-xs leading-5 text-(--color-secondary-text) md:grid-cols-2'>
						<p>
							The server seed phrase must match your sync server setup. It lets this app register as
							an allowed device.
						</p>
						<p>
							The sync passphrase encrypts local data before upload. The sync server cannot recover
							it.
						</p>
					</div>
				</motion.div>
			)}
			{syncChoice === 'no' && (
				<a
					href={SYNC_GUIDE_URL}
					target='_blank'
					rel='noopener noreferrer'
					className='inline-flex items-center gap-2 rounded-full border border-(--color-border) px-3 py-2 text-sm text-(--color-link) transition hover:border-(--color-active-text)'
				>
					Open sync server setup guide
					<ExternalLink className='size-4' />
				</a>
			)}
		</div>
	);
}
