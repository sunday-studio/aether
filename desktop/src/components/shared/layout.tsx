import { Outlet, useLocation } from 'react-router';
import { useRegisterShortcuts } from '~/hooks/use-register-shortcuts';
import { ActivityHeatmap } from './activity-heatmap';
import { CommandPalette } from './command-palette';
import { NavigationControl } from './navigation-control';
import { UpdateNotificationListener } from './update-notification';

export const Layout = () => {
	const location = useLocation();
	const isDev = import.meta.env.DEV;
	const { commandPaletteOpen, setCommandPaletteOpen } = useRegisterShortcuts();
	return (
		<div className='relative h-screen w-screen overflow-hidden bg-(--color-background)'>
			<div
				className='flex h-12 items-center justify-center bg-transparent pr-3 pl-14 select-none'
				data-tauri-drag-region
			>
				{isDev && <div className='text-sm'>{location.pathname}</div>}
			</div>
			<ActivityHeatmap />
			<NavigationControl />
			<Outlet />

			<CommandPalette open={commandPaletteOpen} onOpenChange={setCommandPaletteOpen} />
			<UpdateNotificationListener />
		</div>
	);
};
