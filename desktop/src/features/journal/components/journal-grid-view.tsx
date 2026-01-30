import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { Mic } from "lucide-react";
import { useState } from "react";
import { getGetEntriesQueryKey, useGetEntries } from "~/aether-sdk";
import type { Entry } from "~/aether-sdk/models";
import { AudioRecorderModal } from "~/components/shared/audio-recorder-modal";
import { Button } from "~/components/shared/button";
import { showToast } from "~/components/shared/toast-components";
import { useCreateJournalEntry } from "~/hooks/use-create-journal-entry.ts";
import { groupEntriesByTags, sortEntries } from "../journal.domain.ts";
import { JournalGridItem } from "./journal-grid-item.tsx";

export const JournalGridView = () => {
	const { data: entriesResponse } = useGetEntries();
	const { createEntry } = useCreateJournalEntry();
	const queryClient = useQueryClient();
	const entriesQueryKey = getGetEntriesQueryKey();
	const [isRecorderOpen, setIsRecorderOpen] = useState(false);

	// SDK now returns properly typed PaginatedEntries
	const sortedEntries = sortEntries(entriesResponse?.data?.items ?? []);

	const groupedByTags = groupEntriesByTags(sortedEntries);

	const handleSaveAudio = async (audioBlob: Blob, duration: number) => {
		try {
			// First create an entry
			const placeholder =
				'{"root":{"children":[{"children":[],"direction":"ltr","format":"","indent":0,"type":"paragraph","version":1,"textFormat":0,"textStyle":""}],"direction":"ltr","format":"","indent":0,"type":"root","version":1}}';
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
		<div className="h-full overflow-y-scroll bg-neutral-50">
			{/* Header with actions */}
			<div className="sticky top-0 z-10 bg-neutral-50 border-b border-neutral-200 px-6 py-4">
				<div className="max-w-7xl mx-auto flex items-center gap-2">
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
			</div>

			{/* Grid content grouped by tags */}
			<div className="max-w-7xl mx-auto p-6">
				{groupedByTags.size === 0 ? (
					<div className="text-center py-12 text-neutral-500">
						<p>No entries yet. Create your first entry to get started.</p>
					</div>
				) : (
					Array.from(groupedByTags.entries()).map(
						([tagId, { tagName, entries }]) => (
							<div key={tagId} className="mb-8">
								<h2 className="text-lg font-semibold text-neutral-900 mb-4 px-2">
									{tagName}
								</h2>
								<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
									{entries.map((entry) => (
										<JournalGridItem key={entry.id} entry={entry} />
									))}
								</div>
							</div>
						),
					)
				)}
			</div>

			<AudioRecorderModal
				isOpen={isRecorderOpen}
				onOpenChange={setIsRecorderOpen}
				onSave={handleSaveAudio}
			/>
		</div>
	);
};
