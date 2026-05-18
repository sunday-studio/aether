import { listen } from '@tauri-apps/api/event';
import { ArrowUpRight, DownloadIcon, XIcon } from 'lucide-react';
import { useEffect } from 'react';
import { useNavigate } from 'react-router';
import { toast } from 'sonner';
import type { UpdateInfo } from '~/types/updater';

/**
 * Component that listens for update events and shows toast notifications.
 * Should be placed near the root of the app.
 */
export function UpdateNotificationListener() {
	const navigate = useNavigate();

	useEffect(() => {
		const unlisten = listen<UpdateInfo>('update-available', event => {
			const info = event.payload;

			toast.custom(
				id => (
					<UpdateToast
						info={info}
						onViewChanges={() => {
							toast.dismiss(id);
							navigate('/settings');
						}}
						onDismiss={() => toast.dismiss(id)}
					/>
				),
				{
					duration: 15000,
					position: 'bottom-right',
				},
			);
		});

		return () => {
			unlisten.then(fn => fn());
		};
	}, [navigate]);

	return null;
}

interface UpdateToastProps {
	info: UpdateInfo;
	onViewChanges: () => void;
	onDismiss: () => void;
}

function UpdateToast({ info, onViewChanges, onDismiss }: UpdateToastProps) {
	return (
		<div className='min-w-[320px] overflow-hidden rounded-2xl border border-(--color-border) bg-(--color-panel) shadow-2xl'>
			<div className='flex items-start justify-between gap-4 border-b border-(--color-border) bg-(--color-background-secondary) p-4'>
				<div className='flex items-start gap-3'>
					<div className='rounded-full border border-(--color-border) bg-(--color-background) p-2'>
						<DownloadIcon className='size-4 text-(--color-active-text)' />
					</div>
					<div>
						<p className='text-sm font-medium text-(--color-primary-text)'>Update available</p>
						<p className='mt-1 text-xs text-(--color-secondary-text)'>
							v{info.currentVersion}
							{' -> '}v{info.latestVersion}
						</p>
					</div>
				</div>
				<button
					type='button'
					onClick={onDismiss}
					className='rounded-full p-1 text-(--color-secondary-text) transition-colors hover:bg-(--color-background) hover:text-(--color-active-text)'
				>
					<XIcon className='size-4' />
				</button>
			</div>

			<div className='p-4'>
				<p className='text-sm leading-5 text-(--color-secondary-text)'>
					A new version is ready. Review the changes before installing.
				</p>

				<div className='mt-4 flex items-center gap-2'>
					<button
						type='button'
						onClick={onViewChanges}
						className='inline-flex flex-1 items-center justify-center gap-1.5 rounded-full bg-(--color-active-text) px-3 py-2 text-xs font-medium text-(--color-background) transition-opacity hover:opacity-90'
					>
						View update
						<ArrowUpRight className='size-3.5' />
					</button>
					<button
						type='button'
						onClick={onDismiss}
						className='rounded-full px-3 py-2 text-xs text-(--color-secondary-text) transition-colors hover:bg-(--color-background-secondary) hover:text-(--color-active-text)'
					>
						Later
					</button>
				</div>
			</div>
		</div>
	);
}
