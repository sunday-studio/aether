import { Cloud, KeyRound, UserRound, WandSparkles } from 'lucide-react';
import { FeaturePill } from './feature-pill';

export function IntroStep() {
	return (
		<div className='space-y-6'>
			<div>
				<h3 className='text-3xl font-medium'>Aether is a local-first notebook.</h3>
				<p className='mt-3 max-w-xl text-sm leading-6 text-(--color-secondary-text)'>
					You are about to set up the basics: a name, a recovery seed phrase if you want one,
					optional self-hosted sync, and optional AI keys.
				</p>
			</div>
			<div className='grid gap-3 md:grid-cols-2'>
				<FeaturePill
					icon={UserRound}
					title='Personal'
					copy='Name the workspace without creating an account.'
				/>
				<FeaturePill
					icon={KeyRound}
					title='Recoverable'
					copy='Generate a phrase you can store outside the app.'
				/>
				<FeaturePill
					icon={Cloud}
					title='Self-hosted'
					copy='Bring your own sync server or skip it.'
				/>
				<FeaturePill
					icon={WandSparkles}
					title='AI optional'
					copy='Use hosted AI only when you provide a key.'
				/>
			</div>
		</div>
	);
}
