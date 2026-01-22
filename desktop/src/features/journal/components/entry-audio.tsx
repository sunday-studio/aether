import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AudioPlayer } from "~/components/shared/audio-player";

interface MediaItem {
	id: string;
	entityType: string;
	entityId: string;
	mediaType: string;
	filePath: string;
	metadata: {
		duration?: number;
		format?: string;
	};
}

interface AudioTranscription {
	id: string;
	mediaId: string;
	transcriptionText: string;
	status: "pending" | "processing" | "complete" | "failed";
}

interface EntryAudioProps {
	entryId: string;
}

export const EntryAudio = ({ entryId }: EntryAudioProps) => {
	const [mediaItems, setMediaItems] = useState<MediaItem[]>([]);
	const [transcriptions, setTranscriptions] = useState<
		Record<string, AudioTranscription>
	>({});
	const [audioUrls, setAudioUrls] = useState<Record<string, string>>({});
	const pollingIntervalRef = useRef<number | null>(null);

	// Cleanup blob URLs on unmount
	useEffect(() => {
		return () => {
			Object.values(audioUrls).forEach((url) => {
				URL.revokeObjectURL(url);
			});
			if (pollingIntervalRef.current) {
				clearInterval(pollingIntervalRef.current);
			}
		};
	}, [audioUrls]);

	useEffect(() => {
		const fetchMedia = async () => {
			try {
				const items = await invoke<MediaItem[]>("get_media_items_for_entry", {
					entryId,
				});
				const audioItems = items.filter((item) => item.mediaType === "audio");
				setMediaItems(audioItems);

				// Fetch audio data and create blob URLs
				for (const item of audioItems) {
					try {
						const audioData = await invoke<number[]>("get_audio_data", {
							mediaId: item.id,
						});
						const blob = new Blob([new Uint8Array(audioData)], {
							type: "audio/webm",
						});
						const url = URL.createObjectURL(blob);
						setAudioUrls((prev) => ({ ...prev, [item.id]: url }));
					} catch (error) {
						console.error(`Failed to load audio for ${item.id}:`, error);
					}
				}
			} catch (error) {
				console.error("Failed to fetch media items:", error);
			}
		};

		fetchMedia();
	}, [entryId]);

	useEffect(() => {
		if (mediaItems.length === 0) return;

		const fetchTranscriptions = async () => {
			for (const item of mediaItems) {
				try {
					const trans = await invoke<AudioTranscription[]>("get_transcriptions", {
						mediaId: item.id,
					});
					if (trans.length > 0) {
						// Get the active or first transcription
						const active = trans.find((t) => t.status === "complete") || trans[0];
						setTranscriptions((prev) => {
							// Only update if status changed to avoid infinite loops
							const current = prev[item.id];
							if (
								!current ||
								current.status !== active.status ||
								current.transcriptionText !== active.transcriptionText
							) {
								return { ...prev, [item.id]: active };
							}
							return prev;
						});
					}
				} catch (error) {
					console.error(`Failed to fetch transcriptions for ${item.id}:`, error);
				}
			}
		};

		fetchTranscriptions();

		// Set up polling - check every 2 seconds
		if (pollingIntervalRef.current) {
			clearInterval(pollingIntervalRef.current);
		}

		pollingIntervalRef.current = window.setInterval(() => {
			fetchTranscriptions();
		}, 2000);

		return () => {
			if (pollingIntervalRef.current) {
				clearInterval(pollingIntervalRef.current);
				pollingIntervalRef.current = null;
			}
		};
	}, [mediaItems]);

	if (mediaItems.length === 0) {
		return null;
	}

	return (
		<div className="flex flex-col gap-2">
			{mediaItems.map((item) => {
				const transcription = transcriptions[item.id];
				const audioUrl = audioUrls[item.id];

				return (
					<AudioPlayer
						key={item.id}
						mediaId={item.id}
						audioUrl={audioUrl}
						transcriptionStatus={transcription?.status}
						transcriptionText={transcription?.transcriptionText}
					/>
				);
			})}
		</div>
	);
};
