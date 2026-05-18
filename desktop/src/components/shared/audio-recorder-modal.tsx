import { Mic, Square } from 'lucide-react';
import { useRef, useState } from 'react';
import { cn } from '~/utils/cn';
import { Modal, modalContentStyles } from './modal';

interface AudioRecorderModalProps {
	isOpen: boolean;
	onOpenChange: (open: boolean) => void;
	onSave: (audioBlob: Blob, duration: number) => void;
}

const formatTime = (seconds: number) => {
	const mins = Math.floor(seconds / 60);
	const secs = Math.floor(seconds % 60);
	return `${mins}:${secs.toString().padStart(2, '0')}`;
};

export const AudioRecorderModal = ({ isOpen, onOpenChange, onSave }: AudioRecorderModalProps) => {
	const [isRecording, setIsRecording] = useState(false);
	const [hasRecording, setHasRecording] = useState(false);
	const [duration, setDuration] = useState(0);
	const [error, setError] = useState<string | null>(null);
	const mediaRecorderRef = useRef<MediaRecorder | null>(null);
	const chunksRef = useRef<Blob[]>([]);
	const streamRef = useRef<MediaStream | null>(null);
	const durationIntervalRef = useRef<number | null>(null);
	const startTimeRef = useRef<number | null>(null);

	const startRecording = async () => {
		try {
			setError(null);

			// Check if navigator.mediaDevices is available
			if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
				throw new Error(
					'Microphone access is not available. Please ensure your browser/Tauri app has microphone permissions enabled.',
				);
			}

			const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
			streamRef.current = stream;

			const mediaRecorder = new MediaRecorder(stream, {
				mimeType: 'audio/webm;codecs=opus',
			});
			mediaRecorderRef.current = mediaRecorder;
			chunksRef.current = [];
			setHasRecording(false);

			mediaRecorder.ondataavailable = event => {
				if (event.data.size > 0) {
					chunksRef.current.push(event.data);
					setHasRecording(true);
				}
			};

			mediaRecorder.onstop = () => {
				// Duration is already tracked
			};

			mediaRecorder.start();
			setIsRecording(true);
			startTimeRef.current = Date.now();

			// Update duration every second
			durationIntervalRef.current = window.setInterval(() => {
				if (startTimeRef.current) {
					const elapsed = (Date.now() - startTimeRef.current) / 1000;
					setDuration(elapsed);
				}
			}, 100);
		} catch (err) {
			setError(
				err instanceof Error
					? err.message
					: 'Failed to access microphone. Please check permissions.',
			);
		}
	};

	const stopRecording = () => {
		if (mediaRecorderRef.current && isRecording) {
			mediaRecorderRef.current.stop();
			setIsRecording(false);

			if (durationIntervalRef.current) {
				clearInterval(durationIntervalRef.current);
				durationIntervalRef.current = null;
			}

			// Stop all tracks
			if (streamRef.current) {
				streamRef.current.getTracks().forEach(track => {
					track.stop();
				});
				streamRef.current = null;
			}
		}
	};

	const handleSave = () => {
		if (chunksRef.current.length > 0) {
			const audioBlob = new Blob(chunksRef.current, { type: 'audio/webm' });
			onSave(audioBlob, duration);
			// Reset state
			chunksRef.current = [];
			setHasRecording(false);
			setDuration(0);
			setIsRecording(false);
			onOpenChange(false);
		}
	};

	const handleCancel = () => {
		stopRecording();
		chunksRef.current = [];
		setHasRecording(false);
		setDuration(0);
		setIsRecording(false);
		setError(null);
		onOpenChange(false);
	};

	return (
		<Modal isOpen={isOpen} onOpenChange={open => (open ? onOpenChange(true) : handleCancel())}>
			<div className={modalContentStyles}>
				<div className='flex flex-col items-center gap-4'>
					<h2 className='text-lg font-semibold'>Record Audio</h2>

					{error && <div className='rounded bg-red-50 p-2 text-sm text-red-600'>{error}</div>}

					<div className='flex flex-col items-center gap-2'>
						<button
							type='button'
							onClick={isRecording ? stopRecording : startRecording}
							className={cn(
								'flex h-20 w-20 items-center justify-center rounded-full transition-all',
								isRecording
									? 'bg-red-500 text-white hover:bg-red-600'
									: 'bg-neutral-200 text-neutral-700 hover:bg-neutral-300',
							)}
						>
							{isRecording ? <Square className='h-8 w-8' /> : <Mic className='h-8 w-8' />}
						</button>

						<div className='font-mono text-2xl font-semibold'>{formatTime(duration)}</div>

						{isRecording && (
							<div className='flex items-center gap-2 text-sm text-neutral-500'>
								<div className='h-2 w-2 animate-pulse rounded-full bg-red-500' />
								Recording...
							</div>
						)}
					</div>

					<div className='flex w-full gap-2'>
						<button
							type='button'
							onClick={handleCancel}
							className='flex-1 rounded-lg border border-neutral-300 bg-white px-4 py-2 text-sm text-neutral-700 hover:bg-neutral-50'
						>
							Cancel
						</button>
						<button
							type='button'
							onClick={handleSave}
							disabled={!hasRecording || isRecording}
							className='flex-1 rounded-lg bg-neutral-900 px-4 py-2 text-sm text-white hover:bg-neutral-800 disabled:cursor-not-allowed disabled:opacity-50'
						>
							Save
						</button>
					</div>
				</div>
			</div>
		</Modal>
	);
};
