import { invoke } from "@tauri-apps/api/core";
import { useEffect } from "react";

/**
 * Ensures a media_items blob is on disk before use (e.g. before showing an image/video).
 * When sync.media_sync_policy is "on_demand", fetches from the sync server if missing.
 * No-op when policy is "auto" or sync is not configured.
 * Audio already goes through get_audio_data which does this; use this hook for image/video.
 */
export function useEnsureMediaBlob(mediaId: string | null) {
	useEffect(() => {
		if (!mediaId) return;
		invoke("ensure_media_blob", { pathParams: { mediaId } }).catch(() => {});
	}, [mediaId]);
}
