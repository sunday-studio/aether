import { useSync } from "~/hooks/use-sync";
import { Button } from "~/components/shared/button";
import { TextField } from "~/components/shared/text-field";
import { useState } from "react";
import { format } from "date-fns";

export const SyncSection = () => {
	const {
		status,
		mediaSyncPolicy,
		configure,
		syncNow,
		disconnect,
		setMediaSyncPolicy,
		reconnect,
		isSyncing,
		isConfiguring,
		isReconnecting,
		configureError,
		syncError,
		reconnectError,
	} = useSync();

	const [serverUrl, setServerUrl] = useState("");
	const [passphrase, setPassphrase] = useState("");
	const [reconnectPassphrase, setReconnectPassphrase] = useState("");

	const err = configureError || syncError || reconnectError;

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
								{status.connected
									? status.needs_passphrase
										? "Reconnect required"
										: "Connected"
									: "Not configured"}
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

			{status?.needs_passphrase && (
				<div className="rounded-lg border border-(--color-border) bg-(--color-panel) p-4 space-y-3">
					<p className="text-sm text-(--color-secondary-text)">
						Re-enter your passphrase to sync (it is not stored).
					</p>
					<div className="flex gap-2">
						<TextField
							placeholder="Passphrase"
							type="password"
							value={reconnectPassphrase}
							onChange={(v) => setReconnectPassphrase(v)}
						/>
						<Button
							onClick={() => reconnect(reconnectPassphrase)}
							label={isReconnecting ? "Reconnecting…" : "Reconnect"}
							tooltipContent="Reconnect with passphrase"
							isDisabled={reconnectPassphrase.length < 12 || isReconnecting}
						/>
					</div>
				</div>
			)}

			{status?.connected && !status?.needs_passphrase && (
				<div className="flex items-center gap-4">
					<span className="text-sm">Media:</span>
					<label className="flex items-center gap-2 cursor-pointer">
						<input
							type="radio"
							name="media_policy"
							checked={mediaSyncPolicy === "auto"}
							onChange={() => setMediaSyncPolicy("auto")}
						/>
						<span className="text-sm">Auto sync media</span>
					</label>
					<label className="flex items-center gap-2 cursor-pointer">
						<input
							type="radio"
							name="media_policy"
							checked={mediaSyncPolicy === "on_demand"}
							onChange={() => setMediaSyncPolicy("on_demand")}
						/>
						<span className="text-sm">Download as needed</span>
					</label>
				</div>
			)}

			<div className="flex flex-col gap-4">
				<TextField
					label="Server URL"
					placeholder="https://your-sync-server:8080"
					value={serverUrl}
					onChange={(v) => setServerUrl(v)}
				/>
				<TextField
					label="Passphrase"
					placeholder="min 12 characters"
					type="password"
					value={passphrase}
					onChange={(v) => setPassphrase(v)}
				/>

				<div className="flex items-center justify-end gap-4 mt-4">
					{status?.connected && !status?.needs_passphrase && (
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
						isDisabled={
							!status?.connected || status.needs_passphrase || isSyncing
						}
					/>
					<Button
						onClick={() => configure(serverUrl, passphrase)}
						label={isConfiguring ? "Saving…" : "Save"}
						tooltipContent="Save server URL and passphrase"
						isDisabled={!serverUrl.trim() || passphrase.length < 12 || isConfiguring}
					/>
				</div>
			</div>
		</div>
	);
};
