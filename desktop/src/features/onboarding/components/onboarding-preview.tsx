import { Check, Sparkles } from 'lucide-react';
import { motion } from 'motion/react';
import { cn } from '~/utils/cn';
import { type PreviewItem } from '../onboarding.types';

interface OnboardingPreviewProps {
	items: PreviewItem[];
}

export function OnboardingPreview({ items }: OnboardingPreviewProps) {
	return (
		<section className='relative min-h-[520px] overflow-hidden rounded-lg border border-(--color-border) bg-(--color-card) p-8 shadow-2xl shadow-black/10'>
			<div className='absolute inset-0 bg-[radial-gradient(circle_at_20%_18%,rgba(48,164,108,0.16),transparent_30%),radial-gradient(circle_at_80%_68%,rgba(59,130,246,0.12),transparent_28%)]' />
			<motion.div
				aria-hidden
				className='absolute top-16 right-12 size-44 rounded-full border border-(--color-border)'
				animate={{ rotate: 360 }}
				transition={{ duration: 28, repeat: Infinity, ease: 'linear' }}
			>
				<div className='absolute top-0 left-1/2 size-3 -translate-x-1/2 rounded-full bg-(--color-active-text)' />
				<div className='absolute bottom-6 left-5 size-2 rounded-full bg-sky-500' />
			</motion.div>
			<div className='relative z-10 flex h-full min-h-[456px] flex-col justify-between'>
				<div>
					<div className='inline-flex items-center gap-2 rounded-full border border-(--color-border) bg-(--color-background)/80 px-3 py-1 text-xs text-(--color-secondary-text)'>
						<Sparkles className='size-3.5' />
						First launch setup
					</div>
					<h1 className='newsreader-font mt-8 max-w-lg text-5xl leading-[0.96] md:text-7xl'>
						Shape your quiet place before it starts listening.
					</h1>
					<p className='mt-5 max-w-md text-sm leading-6 text-(--color-secondary-text)'>
						Aether stays local by default. This flow names the space, prepares recovery, and
						connects sync if you already run it. AI remains optional.
					</p>
				</div>

				<div className='grid gap-3'>
					{items.map((item, index) => (
						<motion.div
							key={item.label}
							className='flex items-center justify-between rounded-lg border border-(--color-border) bg-(--color-background)/80 px-4 py-3 text-sm backdrop-blur'
							initial={{ opacity: 0, x: -10 }}
							animate={{ opacity: 1, x: 0 }}
							transition={{ delay: index * 0.06 }}
						>
							<div>
								<p className='font-medium'>{item.label}</p>
								<p className='mt-0.5 text-xs text-(--color-secondary-text)'>{item.value}</p>
							</div>
							<span
								className={cn(
									'grid size-6 place-items-center rounded-full border border-(--color-border)',
									item.active && 'border-(--color-active-text) bg-(--color-active-text) text-white',
								)}
							>
								{item.active ? <Check className='size-3.5' /> : index + 1}
							</span>
						</motion.div>
					))}
				</div>
			</div>
		</section>
	);
}
