import { useSync } from "~/hooks/use-sync";
import { Button } from "~/components/shared/button";
import { TextField } from "~/components/shared/text-field";
import { useState } from "react";
import { format } from "date-fns";

export const SyncSection = () => {
	const {
		status,
		configure,
		syncNow,
		disconnect,
		isSyncing,
		isConfiguring,
		configureError,
		syncError,
	} = useSync();

	const [serverUrl, setServerUrl] = useState("");
	const [passphrase, setPassphrase] = useState("");

	const err = configureError || syncError;

	return (
		<div className="space-y-10 max-w-xl">
			<div>
				<h3 className="text-lg font-medium">Sync</h3>
				<p className="text-sm text-(--color-secondary-text)">
					End-to-end encrypted sync with your own server. Deploy the sync server
					(Docker) and enter its URL and a passphrase.{" "}
					<a
						href="https://github.com/sunday-studio/aether/blob/main/sync-server/README.md"
						target="_blank"
						rel="noopener noreferrer"
					>
						Setup guide
					</a>
				</p>
			</div>

			{status && (
				<div className="rounded-lg border border-(--color-border) bg-(--color-panel) p-4 text-sm">
					<div className="flex items-center justify-between gap-4">
						<div>
							<span
								className={
									status.connected
										? "text-(--color-success)"
										: "text-(--color-secondary-text)"
								}
							>
								{status.connected ? "Connected" : "Not configured"}
							</span>
							{status.pending_changes > 0 && (
								<span className="ml-2 text-(--color-secondary-text)">
									{status.pending_changes} pending
								</span>
							)}
						</div>
						{status.last_sync != null && (
							<span className="text-(--color-secondary-text)">
								Last sync: {format(new Date(status.last_sync), "PPp")}
							</span>
						)}
					</div>
				</div>
			)}

			{err && (
				<div className="rounded-lg border border-red-500/50 bg-red-500/10 p-4 text-sm text-red-600 dark:text-red-400">
					{String(err)}
				</div>
			)}

			<div className="flex flex-col gap-4">
				<TextField
					label="Server URL"
					placeholder="https://your-sync-server:8080"
					value={serverUrl}
					onChange={(e) => setServerUrl(e.target.value)}
				/>
				<TextField
					label="Passphrase"
					placeholder="min 12 characters"
					type="password"
					value={passphrase}
					onChange={(e) => setPassphrase(e.target.value)}
				/>

				<div className="flex items-center justify-end gap-4 mt-4">
					{status?.connected && (
						<Button
							onClick={() => disconnect()}
							label="Disconnect"
							variant="destructive"
							tooltipContent="Clear sync configuration"
						/>
					)}
					<Button
						onClick={() => syncNow()}
						label={isSyncing ? "Syncing…" : "Sync now"}
						tooltipContent="Run sync now"
						disabled={!status?.connected || isSyncing}
					/>
					<Button
						onClick={() => configure(serverUrl, passphrase)}
						label={isConfiguring ? "Saving…" : "Save"}
						tooltipContent="Save server URL and passphrase"
						disabled={!serverUrl.trim() || passphrase.length < 12 || isConfiguring}
					/>
				</div>
			</div>
		</div>
	);
};
