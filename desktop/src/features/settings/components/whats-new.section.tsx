import { DownloadIcon, RefreshCwIcon, XIcon } from 'lucide-react';
import { useEffect, useState } from 'react';
import { Button } from '~/components/shared/button';
import { useUpdater } from '~/hooks/use-updater';
import type { UpdatePreferences } from '~/types/updater';

export const WhatsNewSection = () => {
	const {
		checking,
		available,
		downloading,
		progress,
		info,
		error,
		checkForUpdates,
		downloadAndInstall,
		skipVersion,
		getAppVersion,
		getPreferences,
		setPreferences,
	} = useUpdater();

	const [currentVersion, setCurrentVersion] = useState<string | null>(null);
	const [prefs, setPrefs] = useState<UpdatePreferences | null>(null);

	useEffect(() => {
		getAppVersion().then(setCurrentVersion);
		getPreferences().then(setPrefs);
	}, [getAppVersion, getPreferences]);

	const handlePreferenceChange = async (key: keyof UpdatePreferences, value: boolean) => {
		if (!prefs) return;
		const newPrefs = { ...prefs, [key]: value };
		setPrefs(newPrefs);
		await setPreferences(newPrefs);
	};

	const clearSkippedVersions = async () => {
		if (!prefs) return;
		const newPrefs = { ...prefs, skippedVersions: [] };
		setPrefs(newPrefs);
		await setPreferences(newPrefs);
	};

	return (
		<div className='w-full space-y-10'>
			<div>
				<h3 className='text-lg font-medium'>What's New</h3>
				<p className='text-sm text-(--color-secondary-text)'>
					Check for updates and see what's changed in recent releases.
				</p>
			</div>

			{/* Current version */}
			<div className='rounded-lg border border-(--color-border) bg-(--color-panel) p-4'>
				<div className='flex items-center justify-between'>
					<div>
						<p className='text-sm text-(--color-secondary-text)'>Current version</p>
						<p className='text-lg font-medium'>
							{currentVersion ? `v${currentVersion}` : 'Loading...'}
						</p>
					</div>
					<Button
						onClick={checkForUpdates}
						label={checking ? 'Checking...' : 'Check for updates'}
						tooltipContent='Check for new versions'
						isDisabled={checking}
						iconLeft={<RefreshCwIcon className={`size-4 ${checking ? 'animate-spin' : ''}`} />}
					/>
				</div>
			</div>

			{/* Error display */}
			{error && (
				<div className='rounded-lg border border-red-500/50 bg-red-500/10 p-4 text-sm text-red-600 dark:text-red-400'>
					{error}
				</div>
			)}

			{/* Update available */}
			{available && info && (
				<div className='overflow-hidden rounded-lg border border-(--color-border) bg-(--color-panel)'>
					<div className='border-b border-(--color-border) bg-(--color-background-secondary) p-4'>
						<div className='flex items-center justify-between'>
							<div>
								<p className='font-medium'>Update Available</p>
								<p className='text-sm text-(--color-secondary-text)'>
									v{info.currentVersion} → v{info.latestVersion}
								</p>
							</div>
							<div className='flex items-center gap-2'>
								<Button
									onClick={() => skipVersion(info.latestVersion)}
									label='Skip'
									variant='ghost'
									tooltipContent='Skip this version'
									iconLeft={<XIcon className='size-4' />}
								/>
								<Button
									onClick={downloadAndInstall}
									label={
										downloading ? `Downloading... ${Math.round(progress)}%` : 'Download & Install'
									}
									tooltipContent='Download and install update'
									isDisabled={downloading}
									iconLeft={<DownloadIcon className='size-4' />}
								/>
							</div>
						</div>

						{/* Progress bar */}
						{downloading && (
							<div className='mt-3'>
								<div className='h-1.5 overflow-hidden rounded-full bg-(--color-border)'>
									<div
										className='h-full bg-(--color-active-text) transition-all duration-300'
										style={{ width: `${progress}%` }}
									/>
								</div>
							</div>
						)}
					</div>

					{/* Changelog */}
					{info.changelog && (
						<div className='p-4'>
							<p className='mb-2 text-sm font-medium'>Release Notes</p>
							<div className='prose prose-sm dark:prose-invert max-w-none text-sm text-(--color-secondary-text)'>
								<MarkdownContent content={info.changelog} />
							</div>
						</div>
					)}
				</div>
			)}

			{/* No update available message */}
			{!available && !checking && !error && (
				<div className='rounded-lg border border-(--color-border) bg-(--color-panel) p-4'>
					<p className='text-sm text-(--color-secondary-text)'>
						You're running the latest version. Check back later for updates.
					</p>
				</div>
			)}

			{/* Update preferences */}
			{prefs && (
				<div className='space-y-4'>
					<h4 className='text-sm font-medium'>Update Settings</h4>

					<label className='flex cursor-pointer items-center gap-3'>
						<input
							type='checkbox'
							checked={prefs.autoCheck}
							onChange={e => handlePreferenceChange('autoCheck', e.target.checked)}
							className='rounded border-neutral-400'
						/>
						<div>
							<p className='text-sm'>Automatic update checks</p>
							<p className='text-xs text-(--color-secondary-text)'>
								Check for updates when the app gains focus
							</p>
						</div>
					</label>

					<label className='flex cursor-pointer items-center gap-3'>
						<input
							type='checkbox'
							checked={prefs.autoDownload}
							onChange={e => handlePreferenceChange('autoDownload', e.target.checked)}
							className='rounded border-neutral-400'
						/>
						<div>
							<p className='text-sm'>Auto-download updates</p>
							<p className='text-xs text-(--color-secondary-text)'>
								Download updates in the background (still requires confirmation to install)
							</p>
						</div>
					</label>

					{prefs.skippedVersions.length > 0 && (
						<div className='pt-2'>
							<p className='mb-2 text-sm text-(--color-secondary-text)'>
								Skipped versions: {prefs.skippedVersions.join(', ')}
							</p>
							<Button
								onClick={clearSkippedVersions}
								label='Clear skipped versions'
								variant='ghost'
								tooltipContent='Remove all skipped versions'
							/>
						</div>
					)}
				</div>
			)}
		</div>
	);
};

/** Simple markdown renderer for changelogs */
function MarkdownContent({ content }: { content: string }) {
	// Basic markdown to HTML conversion
	const html = content
		// Headers
		.replace(/^### (.*$)/gim, '<h4 class="font-medium mt-3 mb-1">$1</h4>')
		.replace(/^## (.*$)/gim, '<h3 class="font-medium mt-4 mb-2">$1</h3>')
		.replace(/^# (.*$)/gim, '<h2 class="font-semibold mt-4 mb-2">$1</h2>')
		// Bold
		.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
		// Italic
		.replace(/\*(.+?)\*/g, '<em>$1</em>')
		// List items
		.replace(/^- (.+)$/gim, '<li class="ml-4">• $1</li>')
		// Line breaks
		.replace(/\n\n/g, '<br/><br/>')
		.replace(/\n/g, '<br/>');

	return (
		<div
			// biome-ignore lint/security/noDangerouslySetInnerHtml: Changelog from trusted source
			dangerouslySetInnerHTML={{ __html: html }}
		/>
	);
}
