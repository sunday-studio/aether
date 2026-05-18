import { Antenna, CloudIcon, Palette, WandSparkles } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useSearchParams } from 'react-router';
import { AiSection } from './components/ai.section';
import { PreferencesSection } from './components/preferences.section';
import { SyncSection } from './components/sync.section';
import { WhatsNewSection } from './components/whats-new.section';
import { cn } from '~/utils/cn';
import { RadialAvatar } from '~/components/shared/radiant-avatar';
import { useSettingsStore } from '~/store/settings-store';

type AvailableSections = 'preferences' | 'ai' | 'whats-new' | 'sync';

function getAvailableSection(value: string | null): AvailableSections {
	if (value === 'ai' || value === 'whats-new' || value === 'sync') return value;
	return 'preferences';
}

export const SettingsView = () => {
	const { getValue, settings } = useSettingsStore();
	const user = getValue('user.display_name');
	const [searchParams, setSearchParams] = useSearchParams();
	const [selectedSection, setSelectedSection] = useState<AvailableSections>(
		getAvailableSection(searchParams.get('section')),
	);

	useEffect(() => {
		setSelectedSection(getAvailableSection(searchParams.get('section')));
	}, [searchParams]);

	const selectSection = (section: AvailableSections) => {
		setSelectedSection(section);
		setSearchParams(section === 'preferences' ? {} : { section });
	};

	const sections: Record<
		AvailableSections,
		{ label: string; icon: React.ReactNode; component: React.ReactNode }
	> = {
		preferences: {
			label: 'Preferences',
			icon: <Palette className='size-4' strokeWidth={2.5} />,
			component: <PreferencesSection />,
		},
		sync: {
			label: 'Sync',
			icon: <CloudIcon className='size-4' strokeWidth={2.5} />,
			component: <SyncSection />,
		},
		ai: {
			label: 'AI',
			icon: <WandSparkles className='size-4' strokeWidth={2.5} />,
			component: <AiSection />,
		},
		'whats-new': {
			label: "What's new",
			icon: <Antenna className='size-4' strokeWidth={2.5} />,
			component: <WhatsNewSection />,
		},
	};
	return (
		<div className='mx-auto flex h-[calc(100vh-3rem)] w-full flex-col items-center justify-start gap-5 overflow-y-auto px-4 pt-2 pb-24'>
			<div className='flex flex-col items-center justify-center gap-2'>
				<RadialAvatar size={40} seed={user} />
				<p className='text-sm text-(--color-secondary-text)'>{user}</p>
			</div>
			<div className='mx-auto flex w-full items-center justify-center gap-1'>
				{Object.entries(sections).map(([section, { label, icon }]) => {
					const isSelected = selectedSection === section;
					return (
						<button
							key={section}
							onClick={() => selectSection(section as AvailableSections)}
							type='button'
							className={cn(
								'flex h-8 cursor-pointer items-center gap-2 rounded-full px-2.5 text-sm text-(--color-secondary-text) hover:text-(--color-secondary-text-hover)',
								{
									// text-(--color-navigation-control-active-foreground)
									'bg-(--color-navigation-control-active) text-white inset-shadow-sm inset-shadow-green-600/50 hover:text-(--color-navigation-control-active-foreground)':
										isSelected,
									'bg-neutral-100 hover:text-(--color-navigation-control-foreground)': !isSelected,
								},
							)}
						>
							{icon}
							<p className='text-xs'>{label}</p>
						</button>
					);
				})}
			</div>
			<div className='mx-auto mt-10 w-full max-w-2xl items-center justify-center bg-transparent'>
				{sections[selectedSection as AvailableSections].component}
			</div>
		</div>
	);
};
