import { Play, Pause, Loader } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { cn } from '~/utils/cn';

interface AudioPlayerProps {
	audioUrl?: string;
	transcriptionStatus?: 'pending' | 'processing' | 'complete' | 'failed';
	transcriptionText?: string;
}

const formatTime = (seconds: number) => {
	if (isNaN(seconds)) return '0:00';
	const mins = Math.floor(seconds / 60);
	const secs = Math.floor(seconds % 60);
	return `${mins}:${secs.toString().padStart(2, '0')}`;
};

export const AudioPlayer = ({
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
			console.error('Audio playback error');
		};

		audio.addEventListener('timeupdate', updateTime);
		audio.addEventListener('loadedmetadata', updateDuration);
		audio.addEventListener('ended', handleEnded);
		audio.addEventListener('error', handleError);

		return () => {
			audio.removeEventListener('timeupdate', updateTime);
			audio.removeEventListener('loadedmetadata', updateDuration);
			audio.removeEventListener('ended', handleEnded);
			audio.removeEventListener('error', handleError);
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

	const progress = duration > 0 ? (currentTime / duration) * 100 : 0;

	return (
		<div className='flex flex-col gap-2 rounded-lg border border-neutral-200 bg-neutral-50 p-3'>
			{audioUrl && <audio ref={audioRef} src={audioUrl} preload='metadata' />}

			<div className='flex items-center gap-3'>
				<button
					type='button'
					onClick={togglePlay}
					disabled={!audioUrl}
					className={cn(
						'flex h-10 w-10 items-center justify-center rounded-full',
						'bg-neutral-900 text-white hover:bg-neutral-800',
						'disabled:cursor-not-allowed disabled:opacity-50',
					)}
				>
					{isPlaying ? <Pause className='h-5 w-5' /> : <Play className='ml-0.5 h-5 w-5' />}
				</button>

				<div className='flex flex-1 flex-col gap-1'>
					<div className='relative h-2 overflow-hidden rounded-full bg-neutral-200'>
						<div
							className='absolute top-0 left-0 h-full bg-neutral-900 transition-all'
							style={{ width: `${progress}%` }}
						/>
					</div>
					<div className='flex items-center justify-between text-xs text-neutral-500'>
						<span>{formatTime(currentTime)}</span>
						<span>{formatTime(duration)}</span>
					</div>
				</div>
			</div>

			{(transcriptionStatus === 'pending' || transcriptionStatus === 'processing') && (
				<div className='flex items-center gap-2 text-sm text-neutral-600'>
					<Loader className='h-4 w-4 animate-spin' />
					<span>
						{transcriptionStatus === 'pending'
							? 'Transcription is queued...'
							: 'We are transcribing...'}
					</span>
				</div>
			)}

			{transcriptionStatus === 'complete' && transcriptionText && (
				<div className='mt-2 rounded border border-neutral-200 bg-white p-2'>
					<p className='text-sm whitespace-pre-wrap text-neutral-700'>{transcriptionText}</p>
				</div>
			)}

			{transcriptionStatus === 'failed' && (
				<div className='text-sm text-red-600'>Transcription failed. Please try again.</div>
			)}
		</div>
	);
};
