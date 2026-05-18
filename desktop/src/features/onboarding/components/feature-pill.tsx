import { type LucideIcon } from 'lucide-react';

interface FeaturePillProps {
	icon: LucideIcon;
	title: string;
	copy: string;
}

export function FeaturePill({ icon: Icon, title, copy }: FeaturePillProps) {
	return (
		<div className='rounded-lg border border-(--color-border) bg-(--color-background) p-4'>
			<Icon className='mb-4 size-5 text-(--color-active-text)' />
			<p className='text-sm font-medium'>{title}</p>
			<p className='mt-1 text-xs leading-5 text-(--color-secondary-text)'>{copy}</p>
		</div>
	);
}
