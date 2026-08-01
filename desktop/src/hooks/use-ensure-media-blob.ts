import { invoke } from "@tauri-apps/api/core";
import { useEffect } from "react";

/**
 * Ensures an image or video media_items blob is on disk before use.
 * When sync.media_sync_policy is "on_demand", fetches from the sync server if missing.
 * No-op when policy is "auto" or sync is not configured.
 */
export function useEnsureMediaBlob(mediaId: string | null) {
	useEffect(() => {
		if (!mediaId) return;
		invoke("ensure_media_blob", { pathParams: { mediaId } }).catch(() => {});
	}, [mediaId]);
}
