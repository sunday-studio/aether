import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { Mic } from "lucide-react";
import { useState } from "react";
import { getGetEntriesQueryKey, useGetEntries } from "~/aether-sdk";
import type { Entry } from "~/aether-sdk/models";
import { AudioRecorderModal } from "~/components/shared/audio-recorder-modal";
import { Button } from "~/components/shared/button";
import { Timeline } from "~/components/shared/timeline";
import { showToast } from "~/components/shared/toast-components";
import { useCreateJournalEntry } from "~/hooks/use-create-journal-entry.ts";
import { sortEntries } from "../journal.domain.ts";
import { JournalTimelineItem } from "./journal-timeline-item.tsx";

const placeholder =
	'{"root":{"children":[{"children":[],"direction":"ltr","format":"","indent":0,"type":"paragraph","version":1,"textFormat":0,"textStyle":""}],"direction":"ltr","format":"","indent":0,"type":"root","version":1}}';

export const JournalTimeline = () => {
	const { data: entriesResponse } = useGetEntries();
	const { createEntry } = useCreateJournalEntry();
	const queryClient = useQueryClient();
	const entriesQueryKey = getGetEntriesQueryKey();
	const [isRecorderOpen, setIsRecorderOpen] = useState(false);

	// SDK now returns properly typed PaginatedEntries
	const sortedEntries = sortEntries(entriesResponse?.data?.items ?? []);

	const handleSaveAudio = async (audioBlob: Blob, duration: number) => {
		try {
			// First create an entry

			const now = new Date();

			// Create entry first
			const entry = await invoke<Entry>("create_entry", {
				requestData: {
					document: placeholder,
					date: now.toISOString(),
				},
			});

			// Convert blob to Uint8Array
			const arrayBuffer = await audioBlob.arrayBuffer();
			const audioData = Array.from(new Uint8Array(arrayBuffer));

			// Save audio recording
			const mediaId = await invoke<string>("save_audio_recording", {
				requestData: {
					entryId: entry.id,
					audioData: audioData,
					duration,
					format: "webm",
					autoTranscribe: true,
				},
			});

			// Start transcription
			await invoke("start_transcription", {
				pathParams: {
					mediaId,
				},
			});

			// Refresh entries
			queryClient.invalidateQueries({ queryKey: entriesQueryKey });

			showToast({
				title: "Audio recorded and transcription started",
			});
		} catch (error) {
			console.error("Failed to save audio:", error);
			showToast({
				title: "Failed to save audio recording",
			});
		}
	};

	return (
		<div className="h-full overflow-y-scroll  relative flex justify-center mt-2 mb-100!">
			<Timeline>
				<Timeline.Item
					className="max-w-5xl w-full bg-red-0 pt-6"
					indicatorContainerClassName="w-10"
					leftContainerClassName="w-40"
					rightContent={
						<Timeline.RightContent className="pb-10">
							<div className="flex items-center gap-2">
								<Button
									onClick={createEntry}
									label="Write"
									shortcuts={["⌘", "N"]}
									tooltipContent="Create a new entry"
								/>
								<button
									type="button"
									onClick={() => setIsRecorderOpen(true)}
									className="text-neutral-700 flex items-center gap-1 px-3 py-1.5 text-sm rounded-full bg-neutral-100 hover:ring-neutral-300 ring-3 ring-neutral-200 transition-all duration-200 cursor-pointer"
								>
									<Mic className="w-4 h-4" />
									<span className="text-sm">Record</span>
								</button>
							</div>
						</Timeline.RightContent>
					}
				/>
				{sortedEntries?.map((entry) => {
					return <JournalTimelineItem key={entry.id} entry={entry} />;
				})}
			</Timeline>
			<AudioRecorderModal
				isOpen={isRecorderOpen}
				onOpenChange={setIsRecorderOpen}
				onSave={handleSaveAudio}
			/>
		</div>
	);
};
