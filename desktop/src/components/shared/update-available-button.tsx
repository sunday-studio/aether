import { DownloadIcon } from 'lucide-react';
import { useEffect } from 'react';
import { useNavigate } from 'react-router';
import { useUpdater } from '~/hooks/use-updater';

export function UpdateAvailableButton() {
	const navigate = useNavigate();
	const { available, info, checking, checkForUpdates, getPreferences } = useUpdater();

	useEffect(() => {
		getPreferences().then(preferences => {
			if (preferences?.autoCheck) {
				checkForUpdates();
			}
		});
	}, [checkForUpdates, getPreferences]);

	if (!available || !info) return null;

	return (
		<button
			type='button'
			onClick={() => navigate('/settings?section=whats-new')}
			className='absolute top-2 right-3 inline-flex h-8 items-center gap-2 rounded-full border border-(--color-border) bg-(--color-panel)/90 px-3 text-xs font-medium text-(--color-active-text) shadow-sm backdrop-blur transition-colors hover:bg-(--color-background-secondary)'
			aria-label={`New update available, version ${info.latestVersion}`}
			disabled={checking}
		>
			<span className='relative flex size-2'>
				<span className='absolute inline-flex size-full animate-ping rounded-full bg-(--color-active-text) opacity-30' />
				<span className='relative inline-flex size-2 rounded-full bg-(--color-active-text)' />
			</span>
			<DownloadIcon className='size-3.5' />
			New update
		</button>
	);
}
