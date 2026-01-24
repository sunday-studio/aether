import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

export type SyncStatus = {
	connected: boolean;
	pending_changes: number;
	last_sync: number | null;
};

async function getSyncStatus(): Promise<SyncStatus> {
	return invoke<SyncStatus>("get_sync_status");
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

	return {
		status: status ?? null,
		isLoading,
		refetch,
		syncNow: () => syncNowMutation.mutateAsync(),
		configure: (serverUrl: string, passphrase: string) =>
			configureMutation.mutateAsync({ serverUrl, passphrase }),
		disconnect: () => disconnectMutation.mutateAsync(),
		isSyncing: syncNowMutation.isPending,
		isConfiguring: configureMutation.isPending,
		configureError: configureMutation.error,
		syncError: syncNowMutation.error,
	};
}
