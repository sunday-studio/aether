import { useQueryClient } from '@tanstack/react-query';
import { format } from 'date-fns';
import { useState } from 'react';
import {
	getGetSyncStatusQueryKey,
	useConfigureSync,
	useDisconnectSync,
	useGetSetting,
	useGetSyncStatus,
	useReconnectSync,
	useSetSetting,
	useSyncNow,
} from '~/aether-sdk';
import type { SyncStatus } from '~/aether-sdk/models';
import { Button } from '~/components/shared/button';
import { TextField } from '~/components/shared/text-field';

export const SyncSection = () => {
	const queryClient = useQueryClient();
	const syncStatusQueryKey = getGetSyncStatusQueryKey();

	const { data: statusResponse } = useGetSyncStatus({
		query: {
			refetchInterval: 30_000,
		},
	});

	const status = statusResponse?.data as SyncStatus | undefined;

	const { data: mediaPolicyResponse } = useGetSetting(
		{ key: 'sync.media_sync_policy' },
		{
			query: {
				queryKey: ['sync', 'media_policy'],
			},
		},
	);

	const mediaSyncPolicy = (mediaPolicyResponse?.data?.value as 'auto' | 'on_demand') ?? 'on_demand';

	// useEffect(() => {
	// 	const unlisten = listen<SyncStatus>("sync-status", () => {
	// 		queryClient.invalidateQueries({ queryKey: syncStatusQueryKey });
	// 	});
	// 	return () => {
	// 		unlisten.then((fn) => fn());
	// 	};
	// }, [queryClient, syncStatusQueryKey]);

	const syncNowMutation = useSyncNow({
		mutation: {
			onSuccess: () => {
				queryClient.invalidateQueries({ queryKey: syncStatusQueryKey });
			},
		},
	});

	const configureMutation = useConfigureSync({
		mutation: {
			onSuccess: () => {
				queryClient.invalidateQueries({ queryKey: syncStatusQueryKey });
			},
		},
	});

	const disconnectMutation = useDisconnectSync({
		mutation: {
			onSuccess: () => {
				queryClient.invalidateQueries({ queryKey: syncStatusQueryKey });
			},
		},
	});

	const setMediaSyncPolicyMutation = useSetSetting({
		mutation: {
			onSuccess: () => {
				queryClient.invalidateQueries({ queryKey: syncStatusQueryKey });
				queryClient.invalidateQueries({ queryKey: ['sync', 'media_policy'] });
			},
		},
	});

	const reconnectMutation = useReconnectSync({
		mutation: {
			onSuccess: () => {
				queryClient.invalidateQueries({ queryKey: syncStatusQueryKey });
			},
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

	const err = configureError || syncError || reconnectError;
	const errMessage = err ? String(err) : null;

	return (
		<div className='w-full space-y-10'>
			<div>
				<h3 className='text-lg font-medium'>Sync</h3>
				<p className='text-sm text-(--color-secondary-text)'>
					End-to-end encrypted sync with your own server. Deploy the sync server (Docker) and enter
					its URL, the server seed phrase, and your sync passphrase.{' '}
					<a
						href='https://github.com/sunday-studio/aether/blob/main/docs/reference/sync-server-readme.md'
						target='_blank'
						rel='noopener noreferrer'
					>
						Setup guide
					</a>
				</p>
			</div>

			{status && (
				<div className='rounded-lg border border-(--color-border) bg-(--color-panel) p-4 text-sm'>
					<div className='flex items-center justify-between gap-4'>
						<div>
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
									data: { sync_passphrase: reconnectPassphrase },
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
							onChange={() =>
								setMediaSyncPolicyMutation.mutate({
									data: { key: 'sync.media_sync_policy', value: 'auto' },
								})
							}
						/>
						<span className='text-sm'>Auto sync media</span>
					</label>
					<label className='flex cursor-pointer items-center gap-2'>
						<input
							type='radio'
							name='media_policy'
							checked={mediaSyncPolicy === 'on_demand'}
							onChange={() =>
								setMediaSyncPolicyMutation.mutate({
									data: { key: 'sync.media_sync_policy', value: 'on_demand' },
								})
							}
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
				<TextField
					label='Sync Passphrase'
					placeholder='min 12 characters'
					type='password'
					value={syncPassphrase}
					onChange={v => setSyncPassphrase(v)}
				/>

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
								data: {
									server_url: serverUrl,
									server_seed_phrase: serverSeedPhrase,
									sync_passphrase: syncPassphrase,
								},
							})
						}
						label={isConfiguring ? 'Saving…' : 'Save'}
						tooltipContent='Save server URL and sync credentials'
						// isDisabled={!serverUrl.trim() || passphrase.length < 12 || isConfiguring}
					/>
				</div>
			</div>
		</div>
	);
};
