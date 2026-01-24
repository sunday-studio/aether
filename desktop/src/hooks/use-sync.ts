import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

export type SyncStatus = {
	connected: boolean;
	pending_changes: number;
	last_sync: number | null;
	needs_passphrase: boolean;
};

async function getSyncStatus(): Promise<SyncStatus> {
	return invoke<SyncStatus>("get_sync_status");
}

async function getSetting(key: string): Promise<{ key: string; value: string | null }> {
	return invoke("get_setting", { key });
}

async function setSetting(key: string, value: string): Promise<void> {
	return invoke("set_setting", { key, value });
}

async function syncNow(): Promise<SyncStatus> {
	return invoke<SyncStatus>("sync_now");
}

async function configureSync(serverUrl: string, passphrase: string): Promise<void> {
	return invoke("configure_sync", { server_url: serverUrl, passphrase });
}

async function disconnectSync(): Promise<void> {
	return invoke("disconnect_sync");
}

async function reconnectSync(passphrase: string): Promise<SyncStatus> {
	return invoke<SyncStatus>("reconnect_sync", { passphrase });
}

export function useSync() {
	const qc = useQueryClient();
	const {
		data: status,
		isLoading,
		refetch,
	} = useQuery({
		queryKey: ["sync", "status"],
		queryFn: getSyncStatus,
		refetchInterval: 30_000,
	});

	const { data: mediaPolicy } = useQuery({
		queryKey: ["sync", "media_policy"],
		queryFn: () => getSetting("sync.media_sync_policy").then((r) => r.value ?? "on_demand"),
	});

	useEffect(() => {
		const unlisten = listen<SyncStatus>("sync-status", () => {
			qc.invalidateQueries({ queryKey: ["sync", "status"] });
		});
		return () => {
			unlisten.then((fn) => fn());
		};
	}, [qc]);

	const syncNowMutation = useMutation({
		mutationFn: syncNow,
		onSuccess: () => qc.invalidateQueries({ queryKey: ["sync", "status"] }),
	});

	const configureMutation = useMutation({
		mutationFn: ({ serverUrl, passphrase }: { serverUrl: string; passphrase: string }) =>
			configureSync(serverUrl, passphrase),
		onSuccess: () => qc.invalidateQueries({ queryKey: ["sync", "status"] }),
	});

	const disconnectMutation = useMutation({
		mutationFn: disconnectSync,
		onSuccess: () => qc.invalidateQueries({ queryKey: ["sync", "status"] }),
	});

	const setMediaSyncPolicyMutation = useMutation({
		mutationFn: (value: "auto" | "on_demand") =>
			setSetting("sync.media_sync_policy", value),
		onSuccess: () => {
			qc.invalidateQueries({ queryKey: ["sync", "status"] });
			qc.invalidateQueries({ queryKey: ["sync", "media_policy"] });
		},
	});

	const reconnectMutation = useMutation({
		mutationFn: reconnectSync,
		onSuccess: () => qc.invalidateQueries({ queryKey: ["sync", "status"] }),
	});

	return {
		status: status ?? null,
		mediaSyncPolicy: (mediaPolicy as "auto" | "on_demand") ?? "on_demand",
		isLoading,
		refetch,
		syncNow: () => syncNowMutation.mutateAsync(),
		configure: (serverUrl: string, passphrase: string) =>
			configureMutation.mutateAsync({ serverUrl, passphrase }),
		disconnect: () => disconnectMutation.mutateAsync(),
		setMediaSyncPolicy: (value: "auto" | "on_demand") =>
			setMediaSyncPolicyMutation.mutateAsync(value),
		reconnect: (passphrase: string) => reconnectMutation.mutateAsync(passphrase),
		isSyncing: syncNowMutation.isPending,
		isConfiguring: configureMutation.isPending,
		isReconnecting: reconnectMutation.isPending,
		configureError: configureMutation.error,
		syncError: syncNowMutation.error,
		reconnectError: reconnectMutation.error,
	};
}
