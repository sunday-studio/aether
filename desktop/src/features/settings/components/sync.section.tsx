import { useMutation, useQueryClient } from '@tanstack/react-query';
import { format } from 'date-fns';
import { useEffect, useState } from 'react';
import {
	getGetSyncStatusQueryKey,
	configureSync,
	disconnectSync,
	reconnectSync,
	useGetSyncStatus,
	syncNow,
} from '~/aether-sdk';
import type { SyncStatus } from '~/aether-sdk/models';
import { Button } from '~/components/shared/button';
import { TextField } from '~/components/shared/text-field';
import { useSettingsStore } from '~/store/settings-store';

function getErrorMessage(error: unknown) {
	if (typeof error === 'string') {
		return error;
	}
	if (error instanceof Error) {
		return error.message;
	}
	if (error && typeof error === 'object' && 'data' in error) {
		const data = (error as { data?: unknown }).data;
		if (typeof data === 'string') {
			return data;
		}
		if (data && typeof data === 'object' && 'message' in data) {
			return String((data as { message?: unknown }).message);
		}
		if (data && typeof data === 'object' && 'error' in data) {
			return String((data as { error?: unknown }).error);
		}
		if (data && typeof data === 'object') {
			try {
				return JSON.stringify(data);
			} catch {
				return String(data);
			}
		}
	}
	if (error && typeof error === 'object' && 'message' in error) {
		return String((error as { message?: unknown }).message);
	}
	try {
		return JSON.stringify(error);
	} catch {
		return String(error);
	}
}

export const SyncSection = () => {
	const queryClient = useQueryClient();
	const syncStatusQueryKey = getGetSyncStatusQueryKey();
	const { getValue, setValue } = useSettingsStore();
	const [shouldCheckSyncStatus, setShouldCheckSyncStatus] = useState(true);

	const { data: statusResponse } = useGetSyncStatus({
		query: {
			enabled: shouldCheckSyncStatus,
			refetchInterval: query => {
				const status = (query.state.data as { data?: SyncStatus } | undefined)?.data;
				return status?.server_url ? 30_000 : false;
			},
		},
	});

	const status = statusResponse?.data as SyncStatus | undefined;

	useEffect(() => {
		if (status && !status.server_url) {
			setShouldCheckSyncStatus(false);
		}
	}, [status]);

	const mediaSyncPolicy = getValue('sync.media_sync_policy', 'on_demand') as 'auto' | 'on_demand';

	const syncNowMutation = useMutation({
		mutationFn: () => syncNow(),
		onSuccess: () => {
			setShouldCheckSyncStatus(true);
			queryClient.invalidateQueries({ queryKey: syncStatusQueryKey });
		},
	});

	const configureMutation = useMutation({
		mutationFn: configureSync,
		onSuccess: () => {
			setShouldCheckSyncStatus(true);
			queryClient.invalidateQueries({ queryKey: syncStatusQueryKey });
		},
	});

	const disconnectMutation = useMutation({
		mutationFn: () => disconnectSync(),
		onSuccess: () => {
			setShouldCheckSyncStatus(true);
			queryClient.invalidateQueries({ queryKey: syncStatusQueryKey });
		},
	});

	const reconnectMutation = useMutation({
		mutationFn: reconnectSync,
		onSuccess: () => {
			setShouldCheckSyncStatus(true);
			queryClient.invalidateQueries({ queryKey: syncStatusQueryKey });
		},
	});

	const isSyncing = syncNowMutation.isPending;
	const isConfiguring = configureMutation.isPending;
	const isReconnecting = reconnectMutation.isPending;
	const configureError = configureMutation.error;
	const syncError = syncNowMutation.error;
	const reconnectError = reconnectMutation.error;

	const [serverUrl, setServerUrl] = useState('');
	const [serverSeedPhrase, setServerSeedPhrase] = useState('');
	const [syncPassphrase, setSyncPassphrase] = useState('');
	const [reconnectPassphrase, setReconnectPassphrase] = useState('');
	const canSaveSyncConfig =
		serverUrl.trim().length > 0 &&
		serverSeedPhrase.trim().length >= 12 &&
		syncPassphrase.trim().length >= 12 &&
		!isConfiguring;

	const err = configureError || syncError || reconnectError;
	const errMessage = err ? getErrorMessage(err) : null;

	return (
		<div className='w-full space-y-10'>
			<div>
				<h3 className='text-lg font-medium'>Sync</h3>
				<p className='text-sm text-(--color-secondary-text)'>
					End-to-end encrypted sync with your own server. Deploy the sync server (Docker) and enter
					its URL, the server seed phrase used to enroll devices, and your private sync passphrase
					for encrypting local payloads.{' '}
					<a
						href='https://github.com/sunday-studio/aether/blob/main/docs/reference/sync-server-readme.md'
						target='_blank'
						rel='noopener noreferrer'
						className='text-(--color-link)'
					>
						Setup guide
					</a>
				</p>
			</div>

			{status && (
				<div className='rounded-2xl border border-(--color-border) bg-(--color-panel) p-4 text-sm'>
					<div className='flex items-center justify-between gap-4'>
						<div className=''>
							<span
								className={
									status.connected ? 'text-(--color-success)' : 'text-(--color-secondary-text)'
								}
							>
								{status.connected
									? status.needs_passphrase
										? 'Reconnect required'
										: 'Connected'
									: 'Not configured'}
							</span>
							{status.pending_changes > 0 && (
								<span className='ml-2 text-(--color-secondary-text)'>
									{status.pending_changes} pending
								</span>
							)}
						</div>
						{status.last_sync != null && (
							<span className='text-(--color-secondary-text)'>
								Last sync: {format(new Date(status.last_sync), 'PPp')}
							</span>
						)}
					</div>
				</div>
			)}

			{errMessage && (
				<div className='rounded-lg border border-red-500/50 bg-red-500/10 p-4 text-sm text-red-600 dark:text-red-400'>
					{errMessage}
				</div>
			)}

			{status?.needs_passphrase && (
				<div className='space-y-3 rounded-lg border border-(--color-border) bg-(--color-panel) p-4'>
					<p className='text-sm text-(--color-secondary-text)'>
						Re-enter your sync passphrase to sync.
					</p>
					<div className='flex gap-2'>
						<TextField
							placeholder='Passphrase'
							type='password'
							value={reconnectPassphrase}
							onChange={v => setReconnectPassphrase(v)}
						/>
						<Button
							onClick={() =>
								reconnectMutation.mutate({
									sync_passphrase: reconnectPassphrase,
								})
							}
							label={isReconnecting ? 'Reconnecting…' : 'Reconnect'}
							tooltipContent='Reconnect with passphrase'
							isDisabled={reconnectPassphrase.length < 12 || isReconnecting}
						/>
					</div>
				</div>
			)}

			{status?.connected && !status?.needs_passphrase && (
				<div className='flex items-center gap-4'>
					<span className='text-sm'>Media:</span>
					<label className='flex cursor-pointer items-center gap-2'>
						<input
							type='radio'
							name='media_policy'
							checked={mediaSyncPolicy === 'auto'}
							onChange={() => {
								void setValue('sync.media_sync_policy', 'auto').then(() =>
									queryClient.invalidateQueries({ queryKey: syncStatusQueryKey }),
								);
							}}
						/>
						<span className='text-sm'>Auto sync media</span>
					</label>
					<label className='flex cursor-pointer items-center gap-2'>
						<input
							type='radio'
							name='media_policy'
							checked={mediaSyncPolicy === 'on_demand'}
							onChange={() => {
								void setValue('sync.media_sync_policy', 'on_demand').then(() =>
									queryClient.invalidateQueries({ queryKey: syncStatusQueryKey }),
								);
							}}
						/>
						<span className='text-sm'>Download as needed</span>
					</label>
				</div>
			)}

			<div className='flex flex-col gap-4'>
				<TextField
					label='Server URL'
					placeholder='https://your-sync-server:8080'
					value={serverUrl}
					onChange={v => setServerUrl(v)}
				/>
				<TextField
					label='Server Seed Phrase'
					placeholder='min 12 characters'
					type='password'
					value={serverSeedPhrase}
					onChange={v => setServerSeedPhrase(v)}
				/>
				<p className='-mt-2 text-xs text-(--color-secondary-text)'>
					This must match the seed phrase configured on the sync server. It registers this device
					with the server; it is not your data encryption passphrase.
				</p>
				<TextField
					label='Sync Passphrase'
					placeholder='min 12 characters'
					type='password'
					value={syncPassphrase}
					onChange={v => setSyncPassphrase(v)}
				/>
				<p className='-mt-2 text-xs text-(--color-secondary-text)'>
					This stays personal to your devices and protects the data before it reaches the server.
					Keep it somewhere safe because the server cannot recover it.
				</p>

				<div className='mt-4 flex items-center justify-end gap-4'>
					{status?.connected && !status?.needs_passphrase && (
						<Button
							onClick={() => disconnectMutation.mutate(undefined)}
							label='Disconnect'
							variant='destructive'
							tooltipContent='Clear sync configuration'
						/>
					)}
					<Button
						onClick={() => syncNowMutation.mutate(undefined)}
						label={isSyncing ? 'Syncing…' : 'Sync now'}
						tooltipContent='Run sync now'
						isDisabled={!status?.connected || status.needs_passphrase || isSyncing}
					/>
					<Button
						onClick={() =>
							configureMutation.mutate({
								server_url: serverUrl,
								server_seed_phrase: serverSeedPhrase,
								sync_passphrase: syncPassphrase,
							})
						}
						label={isConfiguring ? 'Saving…' : 'Save'}
						tooltipContent='Save server URL and sync credentials'
						isDisabled={!canSaveSyncConfig}
					/>
				</div>
			</div>
		</div>
	);
};
