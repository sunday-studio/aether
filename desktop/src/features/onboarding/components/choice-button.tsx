import { CheckCircle2 } from 'lucide-react';
import { cn } from '~/utils/cn';

interface ChoiceButtonProps {
	isSelected: boolean;
	onClick: () => void;
	title: string;
	copy: string;
}

export function ChoiceButton({ isSelected, onClick, title, copy }: ChoiceButtonProps) {
	return (
		<button
			type='button'
			onClick={onClick}
			className={cn(
				'flex min-h-24 items-start justify-between gap-4 rounded-lg border border-(--color-border) bg-(--color-background) p-4 text-left transition hover:border-(--color-active-text)',
				isSelected && 'border-(--color-active-text) bg-(--color-background-secondary)',
			)}
		>
			<span>
				<span className='block text-sm font-medium'>{title}</span>
				<span className='mt-2 block text-xs leading-5 text-(--color-secondary-text)'>{copy}</span>
			</span>
			<span
				className={cn(
					'grid size-6 shrink-0 place-items-center rounded-full border border-(--color-border)',
					isSelected && 'border-(--color-active-text) bg-(--color-active-text) text-white',
				)}
			>
				{isSelected && <CheckCircle2 className='size-4' />}
			</span>
		</button>
	);
}
