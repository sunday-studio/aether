import { Mic, Square } from "lucide-react";
import { useRef, useState } from "react";
import { cn } from "~/utils/cn";
import { Modal, modalContentStyles } from "./modal";

interface AudioRecorderModalProps {
	isOpen: boolean;
	onOpenChange: (open: boolean) => void;
	onSave: (audioBlob: Blob, duration: number) => void;
}

export const AudioRecorderModal = ({
	isOpen,
	onOpenChange,
	onSave,
}: AudioRecorderModalProps) => {
	const [isRecording, setIsRecording] = useState(false);
	const [duration, setDuration] = useState(0);
	const [error, setError] = useState<string | null>(null);
	const mediaRecorderRef = useRef<MediaRecorder | null>(null);
	const chunksRef = useRef<Blob[]>([]);
	const streamRef = useRef<MediaStream | null>(null);
	const durationIntervalRef = useRef<number | null>(null);
	const startTimeRef = useRef<number | null>(null);

	const formatTime = (seconds: number) => {
		const mins = Math.floor(seconds / 60);
		const secs = Math.floor(seconds % 60);
		return `${mins}:${secs.toString().padStart(2, "0")}`;
	};

	const startRecording = async () => {
		try {
			setError(null);

			// Check if navigator.mediaDevices is available
			if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
				throw new Error(
					"Microphone access is not available. Please ensure your browser/Tauri app has microphone permissions enabled.",
				);
			}

			const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
			streamRef.current = stream;

			const mediaRecorder = new MediaRecorder(stream, {
				mimeType: "audio/webm;codecs=opus",
			});
			mediaRecorderRef.current = mediaRecorder;
			chunksRef.current = [];

			mediaRecorder.ondataavailable = (event) => {
				if (event.data.size > 0) {
					chunksRef.current.push(event.data);
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
					: "Failed to access microphone. Please check permissions.",
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
				streamRef.current.getTracks().forEach((track) => {
					track.stop();
				});
				streamRef.current = null;
			}
		}
	};

	const handleSave = () => {
		if (chunksRef.current.length > 0) {
			const audioBlob = new Blob(chunksRef.current, { type: "audio/webm" });
			onSave(audioBlob, duration);
			// Reset state
			chunksRef.current = [];
			setDuration(0);
			setIsRecording(false);
			onOpenChange(false);
		}
	};

	const handleCancel = () => {
		stopRecording();
		chunksRef.current = [];
		setDuration(0);
		setIsRecording(false);
		setError(null);
		onOpenChange(false);
	};

	return (
		<Modal isOpen={isOpen} onOpenChange={onOpenChange}>
			<div className={modalContentStyles}>
				<div className="flex flex-col items-center gap-4">
					<h2 className="text-lg font-semibold">Record Audio</h2>

					{error && (
						<div className="text-sm text-red-600 bg-red-50 p-2 rounded">
							{error}
						</div>
					)}

					<div className="flex flex-col items-center gap-2">
						<button
							type="button"
							onClick={isRecording ? stopRecording : startRecording}
							className={cn(
								"w-20 h-20 rounded-full flex items-center justify-center transition-all",
								isRecording
									? "bg-red-500 hover:bg-red-600 text-white"
									: "bg-neutral-200 hover:bg-neutral-300 text-neutral-700",
							)}
						>
							{isRecording ? (
								<Square className="w-8 h-8" />
							) : (
								<Mic className="w-8 h-8" />
							)}
						</button>

						<div className="text-2xl font-mono font-semibold">
							{formatTime(duration)}
						</div>

						{isRecording && (
							<div className="flex items-center gap-2 text-sm text-neutral-500">
								<div className="w-2 h-2 bg-red-500 rounded-full animate-pulse" />
								Recording...
							</div>
						)}
					</div>

					<div className="flex gap-2 w-full">
						<button
							type="button"
							onClick={handleCancel}
							className="flex-1 px-4 py-2 text-sm rounded-lg border border-neutral-300 bg-white hover:bg-neutral-50 text-neutral-700"
						>
							Cancel
						</button>
						<button
							type="button"
							onClick={handleSave}
							// disabled={chunksRef.current.length === 0 || isRecording}
							className="flex-1 px-4 py-2 text-sm rounded-lg bg-neutral-900 text-white hover:bg-neutral-800 disabled:opacity-50 disabled:cursor-not-allowed"
						>
							Save
						</button>
					</div>
				</div>
			</div>
		</Modal>
	);
};
