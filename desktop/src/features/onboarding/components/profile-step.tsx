import { Copy } from 'lucide-react';
import { TextField } from '~/components/shared/text-field';

interface ProfileStepProps {
	displayName: string;
	recoverySeed: string;
	onDisplayNameChange: (value: string) => void;
	onGenerateRecoverySeed: () => void;
}

export function ProfileStep({
	displayName,
	recoverySeed,
	onDisplayNameChange,
	onGenerateRecoverySeed,
}: ProfileStepProps) {
	return (
		<div className='space-y-6'>
			<div>
				<h3 className='text-3xl font-medium'>Start with a name.</h3>
				<p className='mt-3 max-w-xl text-sm leading-6 text-(--color-secondary-text)'>
					The name is local app context. The recovery phrase is optional, but useful if you want a
					memorable seed stored with your setup.
				</p>
			</div>
			<TextField
				label='Display name'
				placeholder='Ada'
				value={displayName}
				onChange={onDisplayNameChange}
			/>
			<div className='rounded-lg border border-(--color-border) bg-(--color-background) p-4'>
				<div className='flex flex-wrap items-center justify-between gap-3'>
					<div>
						<p className='text-sm font-medium'>Recovery seed phrase</p>
						<p className='mt-1 text-xs text-(--color-secondary-text)'>
							Generate one now, or leave it empty and continue.
						</p>
					</div>
					<div className='flex gap-2'>
						<button
							type='button'
							onClick={onGenerateRecoverySeed}
							className='rounded-full border border-(--color-border) px-3 py-2 text-xs transition hover:border-(--color-active-text)'
						>
							Generate
						</button>
						<button
							type='button'
							onClick={() => recoverySeed && navigator.clipboard?.writeText(recoverySeed)}
							disabled={!recoverySeed}
							className='grid size-8 place-items-center rounded-full border border-(--color-border) text-(--color-secondary-text) transition hover:text-(--color-active-text) disabled:opacity-40'
							aria-label='Copy recovery seed phrase'
						>
							<Copy className='size-4' />
						</button>
					</div>
				</div>
				<p className='mt-4 min-h-12 rounded-lg bg-(--color-card) p-3 text-sm leading-6 text-(--color-secondary-text)'>
					{recoverySeed || 'No seed phrase generated yet.'}
				</p>
			</div>
		</div>
	);
}
