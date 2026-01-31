import { DownloadIcon, RefreshCwIcon, XIcon } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "~/components/shared/button";
import { useUpdater } from "~/hooks/use-updater";

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
		dismissUpdate,
		getAppVersion,
	} = useUpdater();

	const [currentVersion, setCurrentVersion] = useState<string | null>(null);

	useEffect(() => {
		getAppVersion().then(setCurrentVersion);
	}, [getAppVersion]);

	return (
		<div className="space-y-10 max-w-xl">
			<div>
				<h3 className="text-lg font-medium">What's New</h3>
				<p className="text-sm text-(--color-secondary-text)">
					Check for updates and see what's changed in recent releases.
				</p>
			</div>

			{/* Current version */}
			<div className="rounded-lg border border-(--color-border) bg-(--color-panel) p-4">
				<div className="flex items-center justify-between">
					<div>
						<p className="text-sm text-(--color-secondary-text)">
							Current version
						</p>
						<p className="text-lg font-medium">
							{currentVersion ? `v${currentVersion}` : "Loading..."}
						</p>
					</div>
					<Button
						onClick={checkForUpdates}
						label={checking ? "Checking..." : "Check for updates"}
						tooltipContent="Check for new versions"
						isDisabled={checking}
						iconLeft={
							<RefreshCwIcon
								className={`size-4 ${checking ? "animate-spin" : ""}`}
							/>
						}
					/>
				</div>
			</div>

			{/* Error display */}
			{error && (
				<div className="rounded-lg border border-red-500/50 bg-red-500/10 p-4 text-sm text-red-600 dark:text-red-400">
					{error}
				</div>
			)}

			{/* Update available */}
			{available && info && (
				<div className="rounded-lg border border-(--color-border) bg-(--color-panel) overflow-hidden">
					<div className="p-4 bg-(--color-background-secondary) border-b border-(--color-border)">
						<div className="flex items-center justify-between">
							<div>
								<p className="font-medium">Update Available</p>
								<p className="text-sm text-(--color-secondary-text)">
									v{info.currentVersion} → v{info.latestVersion}
								</p>
							</div>
							<div className="flex items-center gap-2">
								<Button
									onClick={() => skipVersion(info.latestVersion)}
									label="Skip"
									variant="ghost"
									tooltipContent="Skip this version"
									iconLeft={<XIcon className="size-4" />}
								/>
								<Button
									onClick={downloadAndInstall}
									label={downloading ? `Downloading... ${Math.round(progress)}%` : "Download & Install"}
									tooltipContent="Download and install update"
									isDisabled={downloading}
									iconLeft={<DownloadIcon className="size-4" />}
								/>
							</div>
						</div>

						{/* Progress bar */}
						{downloading && (
							<div className="mt-3">
								<div className="h-1.5 bg-(--color-border) rounded-full overflow-hidden">
									<div
										className="h-full bg-(--color-active-text) transition-all duration-300"
										style={{ width: `${progress}%` }}
									/>
								</div>
							</div>
						)}
					</div>

					{/* Changelog */}
					{info.changelog && (
						<div className="p-4">
							<p className="text-sm font-medium mb-2">Release Notes</p>
							<div className="text-sm text-(--color-secondary-text) prose prose-sm dark:prose-invert max-w-none">
								<MarkdownContent content={info.changelog} />
							</div>
						</div>
					)}
				</div>
			)}

			{/* No update available message */}
			{!available && !checking && !error && (
				<div className="rounded-lg border border-(--color-border) bg-(--color-panel) p-4">
					<p className="text-sm text-(--color-secondary-text)">
						You're running the latest version. Check back later for updates.
					</p>
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
		.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
		// Italic
		.replace(/\*(.+?)\*/g, "<em>$1</em>")
		// List items
		.replace(/^- (.+)$/gim, '<li class="ml-4">• $1</li>')
		// Line breaks
		.replace(/\n\n/g, "<br/><br/>")
		.replace(/\n/g, "<br/>");

	return (
		<div
			// biome-ignore lint/security/noDangerouslySetInnerHtml: Changelog from trusted source
			dangerouslySetInnerHTML={{ __html: html }}
		/>
	);
}
