import { Play, Pause, Loader } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { cn } from "~/utils/cn";

interface AudioPlayerProps {
	mediaId: string;
	audioUrl?: string;
	transcriptionStatus?: "pending" | "processing" | "complete" | "failed";
	transcriptionText?: string;
}

export const AudioPlayer = ({
	mediaId,
	audioUrl,
	transcriptionStatus,
	transcriptionText,
}: AudioPlayerProps) => {
	const [isPlaying, setIsPlaying] = useState(false);
	const [duration, setDuration] = useState(0);
	const [currentTime, setCurrentTime] = useState(0);
	const audioRef = useRef<HTMLAudioElement | null>(null);

	useEffect(() => {
		const audio = audioRef.current;
		if (!audio || !audioUrl) return;

		// Reset state when URL changes
		setIsPlaying(false);
		setCurrentTime(0);
		setDuration(0);
		audio.pause();
		audio.currentTime = 0;

		const updateTime = () => setCurrentTime(audio.currentTime);
		const updateDuration = () => setDuration(audio.duration);
		const handleEnded = () => setIsPlaying(false);
		const handleError = () => {
			setIsPlaying(false);
			console.error("Audio playback error");
		};

		audio.addEventListener("timeupdate", updateTime);
		audio.addEventListener("loadedmetadata", updateDuration);
		audio.addEventListener("ended", handleEnded);
		audio.addEventListener("error", handleError);

		return () => {
			audio.removeEventListener("timeupdate", updateTime);
			audio.removeEventListener("loadedmetadata", updateDuration);
			audio.removeEventListener("ended", handleEnded);
			audio.removeEventListener("error", handleError);
		};
	}, [audioUrl]);

	const togglePlay = () => {
		const audio = audioRef.current;
		if (!audio) return;

		if (isPlaying) {
			audio.pause();
		} else {
			audio.play();
		}
		setIsPlaying(!isPlaying);
	};

	const formatTime = (seconds: number) => {
		if (isNaN(seconds)) return "0:00";
		const mins = Math.floor(seconds / 60);
		const secs = Math.floor(seconds % 60);
		return `${mins}:${secs.toString().padStart(2, "0")}`;
	};

	const progress = duration > 0 ? (currentTime / duration) * 100 : 0;

	return (
		<div className="flex flex-col gap-2 p-3 bg-neutral-50 rounded-lg border border-neutral-200">
			{audioUrl && (
				<audio ref={audioRef} src={audioUrl} preload="metadata" />
			)}

			<div className="flex items-center gap-3">
				<button
					type="button"
					onClick={togglePlay}
					disabled={!audioUrl}
					className={cn(
						"w-10 h-10 rounded-full flex items-center justify-center",
						"bg-neutral-900 text-white hover:bg-neutral-800",
						"disabled:opacity-50 disabled:cursor-not-allowed",
					)}
				>
					{isPlaying ? (
						<Pause className="w-5 h-5" />
					) : (
						<Play className="w-5 h-5 ml-0.5" />
					)}
				</button>

				<div className="flex-1 flex flex-col gap-1">
					<div className="relative h-2 bg-neutral-200 rounded-full overflow-hidden">
						<div
							className="absolute top-0 left-0 h-full bg-neutral-900 transition-all"
							style={{ width: `${progress}%` }}
						/>
					</div>
					<div className="flex items-center justify-between text-xs text-neutral-500">
						<span>{formatTime(currentTime)}</span>
						<span>{formatTime(duration)}</span>
					</div>
				</div>
			</div>

			{transcriptionStatus === "processing" && (
				<div className="flex items-center gap-2 text-sm text-neutral-600">
					<Loader className="w-4 h-4 animate-spin" />
					<span>We are transcribing...</span>
				</div>
			)}

			{transcriptionStatus === "complete" && transcriptionText && (
				<div className="mt-2 p-2 bg-white rounded border border-neutral-200">
					<p className="text-sm text-neutral-700 whitespace-pre-wrap">
						{transcriptionText}
					</p>
				</div>
			)}

			{transcriptionStatus === "failed" && (
				<div className="text-sm text-red-600">
					Transcription failed. Please try again.
				</div>
			)}
		</div>
	);
};
