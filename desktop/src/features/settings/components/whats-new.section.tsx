import { DownloadIcon, RefreshCwIcon, RotateCcwIcon, SparklesIcon, XIcon } from 'lucide-react';
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
	const [lastCheckMessage, setLastCheckMessage] = useState<string | null>(null);

	useEffect(() => {
		getAppVersion().then(setCurrentVersion);
		getPreferences().then(setPrefs);
	}, [getAppVersion, getPreferences]);

	const handleCheck = async () => {
		setLastCheckMessage(null);
		const updateInfo = await checkForUpdates();
		setLastCheckMessage(
			updateInfo ? `Version ${updateInfo.latestVersion} is ready.` : 'You are up to date.',
		);
	};

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

	const versionLine = info
		? `v${info.currentVersion} -> v${info.latestVersion}`
		: currentVersion
			? `v${currentVersion}`
			: 'Loading version...';

	return (
		<div className='w-full space-y-8'>
			<div>
				<p className='text-xs font-medium tracking-[0.18em] text-(--color-secondary-text) uppercase'>
					Release channel
				</p>
				<h3 className='mt-2 text-2xl font-semibold'>What's new</h3>
				<p className='mt-2 text-sm text-(--color-secondary-text)'>
					Check updates and release notes.
				</p>
			</div>

			<section className='overflow-hidden rounded-3xl border border-(--color-border) bg-(--color-panel) shadow-sm'>
				<div className='relative border-b border-(--color-border) bg-(--color-background-secondary) p-6'>
					<div className='absolute inset-0 opacity-40 [background:radial-gradient(circle_at_20%_10%,var(--color-active-text),transparent_28%),radial-gradient(circle_at_90%_20%,var(--color-border),transparent_24%)]' />
					<div className='relative flex flex-col gap-5 sm:flex-row sm:items-start sm:justify-between'>
						<div className='space-y-3'>
							<div className='inline-flex items-center gap-2 rounded-full border border-(--color-border) bg-(--color-background)/70 px-3 py-1 text-xs text-(--color-secondary-text)'>
								<SparklesIcon className='size-3.5' />
								{available && info ? 'Update available' : 'Aether is current'}
							</div>
							<div>
								<h4 className='text-xl font-semibold'>
									{available && info ? 'Ready to install' : 'Aether Desktop'}
								</h4>
								<p className='mt-1 text-sm text-(--color-secondary-text)'>{versionLine}</p>
								{info?.publishedAt && (
									<p className='mt-1 text-xs text-(--color-secondary-text)'>
										Published {formatPublishedAt(info.publishedAt)}
									</p>
								)}
							</div>
						</div>

						<div className='flex flex-wrap items-center gap-2'>
							<Button
								onClick={handleCheck}
								label={checking ? 'Checking...' : 'Check'}
								tooltipContent='Check for new versions'
								isDisabled={checking || downloading}
								variant='secondary'
								iconLeft={<RefreshCwIcon className={`size-4 ${checking ? 'animate-spin' : ''}`} />}
							/>
							{available && info && (
								<Button
									onClick={downloadAndInstall}
									label={downloading ? `Downloading ${Math.round(progress)}%` : 'Install update'}
									tooltipContent='Download and install update'
									isDisabled={downloading}
									iconLeft={<DownloadIcon className='size-4' />}
								/>
							)}
						</div>
					</div>

					{downloading && (
						<div className='relative mt-6'>
							<div className='h-2 overflow-hidden rounded-full bg-(--color-border)'>
								<div
									className='h-full rounded-full bg-(--color-active-text) transition-all duration-300'
									style={{ width: `${progress}%` }}
								/>
							</div>
						</div>
					)}
				</div>

				{available && info ? (
					<div className='divide-y divide-(--color-border)'>
						<div className='p-6'>
							<div className='mb-4 flex items-center justify-between gap-3'>
								<div>
									<p className='text-sm font-medium'>Release notes</p>
									<p className='text-xs text-(--color-secondary-text)'>Review before updating.</p>
								</div>
								<Button
									onClick={() => skipVersion(info.latestVersion)}
									label='Skip this version'
									variant='ghost'
									tooltipContent='Do not show this version again'
									iconLeft={<XIcon className='size-4' />}
									isDisabled={downloading}
								/>
							</div>
							<ReleaseNotes content={info.changelog} />
						</div>
					</div>
				) : (
					<div className='p-6'>
						<div className='rounded-2xl border border-dashed border-(--color-border) bg-(--color-background) p-5'>
							<p className='text-sm font-medium'>No pending update</p>
							<p className='mt-1 text-sm text-(--color-secondary-text)'>
								{lastCheckMessage ?? 'Check for updates when you are ready.'}
							</p>
						</div>
					</div>
				)}
			</section>

			{error && (
				<div className='rounded-2xl border border-red-500/30 bg-red-500/10 p-4 text-sm text-red-600'>
					{error}
				</div>
			)}

			{prefs && (
				<section className='space-y-4 rounded-3xl border border-(--color-border) bg-(--color-panel) p-5'>
					<div>
						<h4 className='text-sm font-medium'>Update preferences</h4>
						<p className='mt-1 text-xs text-(--color-secondary-text)'>
							Choose how updates are checked.
						</p>
					</div>

					<PreferenceToggle
						label='Automatic update checks'
						description='Check when the app opens.'
						checked={prefs.autoCheck}
						onChange={checked => handlePreferenceChange('autoCheck', checked)}
					/>
					{prefs.skippedVersions.length > 0 && (
						<div className='flex flex-col gap-3 rounded-2xl border border-(--color-border) bg-(--color-background) p-4 sm:flex-row sm:items-center sm:justify-between'>
							<p className='text-sm text-(--color-secondary-text)'>
								Skipped versions: {prefs.skippedVersions.join(', ')}
							</p>
							<Button
								onClick={clearSkippedVersions}
								label='Clear skipped'
								variant='ghost'
								tooltipContent='Remove all skipped versions'
								iconLeft={<RotateCcwIcon className='size-4' />}
							/>
						</div>
					)}
				</section>
			)}
		</div>
	);
};

function PreferenceToggle({
	label,
	description,
	checked,
	onChange,
}: {
	label: string;
	description: string;
	checked: boolean;
	onChange: (checked: boolean) => void;
}) {
	return (
		<label className='flex cursor-pointer items-center justify-between gap-4 rounded-2xl border border-(--color-border) bg-(--color-background) p-4'>
			<span>
				<span className='block text-sm font-medium'>{label}</span>
				<span className='mt-1 block text-xs text-(--color-secondary-text)'>{description}</span>
			</span>
			<input
				type='checkbox'
				checked={checked}
				onChange={event => onChange(event.target.checked)}
				className='size-4 rounded border-(--color-border)'
			/>
		</label>
	);
}

function ReleaseNotes({ content }: { content: string }) {
	if (!content.trim()) {
		return (
			<p className='rounded-2xl border border-(--color-border) bg-(--color-background) p-4 text-sm text-(--color-secondary-text)'>
				This update did not include release notes.
			</p>
		);
	}

	return (
		<div className='max-h-80 space-y-2 overflow-y-auto rounded-2xl border border-(--color-border) bg-(--color-background) p-4'>
			{content.split('\n').map((line, index) => (
				<ReleaseNoteLine key={`${line}-${index}`} line={line} />
			))}
		</div>
	);
}

function ReleaseNoteLine({ line }: { line: string }) {
	const trimmed = line.trim();
	if (!trimmed) return <div className='h-2' />;

	if (trimmed.startsWith('#')) {
		return <p className='pt-2 text-sm font-semibold'>{trimmed.replace(/^#+\s*/, '')}</p>;
	}

	if (trimmed.startsWith('- ') || trimmed.startsWith('* ')) {
		return (
			<p className='pl-3 text-sm leading-6 text-(--color-secondary-text)'>
				<span className='mr-2 text-(--color-active-text)'>•</span>
				{trimmed.slice(2)}
			</p>
		);
	}

	return <p className='text-sm leading-6 text-(--color-secondary-text)'>{trimmed}</p>;
}

function formatPublishedAt(value: string) {
	const date = new Date(value);
	if (Number.isNaN(date.getTime())) return value;
	return date.toLocaleDateString(undefined, {
		month: 'short',
		day: 'numeric',
		year: 'numeric',
	});
}
