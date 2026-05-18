import { motion } from 'motion/react';
import { cn } from '~/utils/cn';
import { steps } from '../onboarding.constants';

interface OnboardingStepHeaderProps {
	stepIndex: number;
	progress: number;
	onStepChange: (index: number) => void;
}

export function OnboardingStepHeader({
	stepIndex,
	progress,
	onStepChange,
}: OnboardingStepHeaderProps) {
	const currentStep = steps[stepIndex];

	return (
		<div className='space-y-5 border-b border-(--color-border) pb-5'>
			<div className='flex items-center justify-between gap-4'>
				<div>
					<p className='text-xs tracking-[0.2em] text-(--color-secondary-text) uppercase'>
						Step {stepIndex + 1} of {steps.length}
					</p>
					<h2 className='mt-1 text-2xl font-medium'>{currentStep.label}</h2>
				</div>
				<div className='flex items-center gap-1'>
					{steps.map((step, index) => {
						const Icon = step.icon;
						return (
							<button
								key={step.id}
								type='button'
								onClick={() => onStepChange(index)}
								className={cn(
									'grid size-9 place-items-center rounded-full border border-(--color-border) text-(--color-secondary-text) transition',
									index === stepIndex &&
										'border-(--color-active-text) bg-(--color-active-text) text-white',
								)}
								aria-label={`Go to ${step.label}`}
							>
								<Icon className='size-4' />
							</button>
						);
					})}
				</div>
			</div>
			<div className='h-1.5 overflow-hidden rounded-full bg-(--color-background-secondary)'>
				<motion.div
					className='h-full rounded-full bg-(--color-active-text)'
					animate={{ width: `${progress}%` }}
					transition={{ type: 'spring', stiffness: 120, damping: 20 }}
				/>
			</div>
		</div>
	);
}
