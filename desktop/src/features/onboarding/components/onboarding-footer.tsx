import { ArrowLeft, ArrowRight } from 'lucide-react';
import { Button } from '~/components/shared/button';
import { type AiChoice } from '../onboarding.types';

interface OnboardingFooterProps {
	stepIndex: number;
	isSaving: boolean;
	isAiStep: boolean;
	aiChoice: AiChoice;
	activeApiKey: string;
	canContinueCurrentStep: boolean;
	statusMessage: string | null;
	errorMessage: string | null;
	onBack: () => void;
	onContinue: () => void;
	onComplete: () => void;
	onValidateProvider: () => void;
	isConfigured: (value: string) => boolean;
}

export function OnboardingFooter({
	stepIndex,
	isSaving,
	isAiStep,
	aiChoice,
	activeApiKey,
	canContinueCurrentStep,
	statusMessage,
	errorMessage,
	onBack,
	onContinue,
	onComplete,
	onValidateProvider,
	isConfigured,
}: OnboardingFooterProps) {
	return (
		<div className='space-y-3 border-t border-(--color-border) pt-5'>
			{statusMessage && (
				<div className='rounded-lg border border-emerald-500/30 bg-emerald-500/10 p-3 text-sm text-emerald-700'>
					{statusMessage}
				</div>
			)}
			{errorMessage && (
				<div className='rounded-lg border border-red-500/30 bg-red-500/10 p-3 text-sm text-red-600'>
					{errorMessage}
				</div>
			)}
			<div className='flex flex-wrap items-center justify-between gap-3'>
				<button
					type='button'
					onClick={onBack}
					disabled={stepIndex === 0 || isSaving}
					className='inline-flex items-center gap-2 text-sm text-(--color-secondary-text) transition hover:text-(--color-active-text) disabled:opacity-40'
				>
					<ArrowLeft className='size-4' />
					Back
				</button>
				<div className='flex flex-wrap items-center justify-end gap-3'>
					{isAiStep && aiChoice === 'yes' && (
						<Button
							onClick={onValidateProvider}
							label={isSaving ? 'Checking...' : 'Validate provider'}
							tooltipContent='Validate the selected provider credentials'
							isDisabled={isSaving || !isConfigured(activeApiKey)}
						/>
					)}
					{isAiStep ? (
						<Button
							onClick={onComplete}
							label={isSaving ? 'Saving...' : 'Start using Aether'}
							tooltipContent='Finish onboarding'
							isDisabled={isSaving || aiChoice === null}
						/>
					) : (
						<button
							type='button'
							onClick={onContinue}
							disabled={isSaving || !canContinueCurrentStep}
							className='inline-flex items-center gap-2 rounded-full bg-(--color-active-text) px-4 py-2.5 text-sm leading-none text-white transition hover:bg-(--color-active-text-hover) disabled:opacity-50'
						>
							Continue
							<ArrowRight className='size-4' />
						</button>
					)}
				</div>
			</div>
		</div>
	);
}
