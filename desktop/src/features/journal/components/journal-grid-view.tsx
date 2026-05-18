import { useGetEntries } from '~/aether-sdk';
import { Button } from '~/components/shared/button';
import { useCreateJournalEntry } from '~/hooks/use-create-journal-entry.ts';
import { groupEntriesByTags, sortEntries } from '../journal.domain.ts';
import { JournalGridItem } from './journal-grid-item.tsx';

export const JournalGridView = () => {
	const { data: entriesResponse } = useGetEntries();
	const { createEntry } = useCreateJournalEntry();

	// SDK now returns properly typed PaginatedEntries
	const sortedEntries = sortEntries(entriesResponse?.data?.items ?? []);

	const groupedByTags = groupEntriesByTags(sortedEntries);

	// Audio recording is intentionally hidden for v1 until the flow is ready to ship.
	// const queryClient = useQueryClient();
	// const [isRecorderOpen, setIsRecorderOpen] = useState(false);
	// const handleSaveAudio = async (audioBlob: Blob, duration: number) => {
	// 	try {
	// 		const placeholder =
	// 			'{"root":{"children":[{"children":[],"direction":"ltr","format":"","indent":0,"type":"paragraph","version":1,"textFormat":0,"textStyle":""}],"direction":"ltr","format":"","indent":0,"type":"root","version":1}}';
	// 		const now = new Date();
	// 		const entry = await invoke<Entry>("create_entry", {
	// 			requestData: {
	// 				document: placeholder,
	// 				date: now.toISOString(),
	// 			},
	// 		});
	// 		const arrayBuffer = await audioBlob.arrayBuffer();
	// 		const audioData = Array.from(new Uint8Array(arrayBuffer));
	// 		const mediaId = await invoke<string>("save_audio_recording", {
	// 			requestData: {
	// 				entryId: entry.id,
	// 				audioData: audioData,
	// 				duration,
	// 				format: "webm",
	// 				autoTranscribe: true,
	// 			},
	// 		});
	// 		await invoke("start_transcription", {
	// 			pathParams: {
	// 				mediaId,
	// 			},
	// 		});
	// 		invalidateEntryQueries(queryClient);
	// 		showToast({
	// 			title: "Audio recorded and transcription started",
	// 		});
	// 	} catch (error) {
	// 		console.error("Failed to save audio:", error);
	// 		showToast({
	// 			title: "Failed to save audio recording",
	// 		});
	// 	}
	// };

	return (
		<div className='h-full overflow-y-scroll bg-neutral-50'>
			{/* Header with actions */}
			<div className='sticky top-0 z-10 border-b border-neutral-200 bg-neutral-50 px-6 py-4'>
				<div className='mx-auto flex max-w-7xl items-center gap-2'>
					<Button
						onClick={createEntry}
						label='Write'
						shortcuts={['⌘', 'N']}
						tooltipContent='Create a new entry'
					/>
					{/* <button
						type="button"
						onClick={() => setIsRecorderOpen(true)}
						className="text-neutral-700 flex items-center gap-1 px-3 py-1.5 text-sm rounded-full bg-neutral-100 hover:ring-neutral-300 ring-3 ring-neutral-200 transition-all duration-200 cursor-pointer"
					>
						<Mic className="w-4 h-4" />
						<span className="text-sm">Record</span>
					</button> */}
				</div>
			</div>

			{/* Grid content grouped by tags */}
			<div className='mx-auto max-w-7xl p-6'>
				{groupedByTags.size === 0 ? (
					<div className='py-12 text-center text-neutral-500'>
						<p>No entries yet. Create your first entry to get started.</p>
					</div>
				) : (
					Array.from(groupedByTags.entries()).map(([tagId, { tagName, entries }]) => (
						<div key={tagId} className='mb-8'>
							<h2 className='mb-4 px-2 text-lg font-semibold text-neutral-900'>{tagName}</h2>
							<div className='grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4'>
								{entries.map(entry => (
									<JournalGridItem key={entry.id} entry={entry} />
								))}
							</div>
						</div>
					))
				)}
			</div>

			{/* <AudioRecorderModal
				isOpen={isRecorderOpen}
				onOpenChange={setIsRecorderOpen}
				onSave={handleSaveAudio}
			/> */}
		</div>
	);
};
