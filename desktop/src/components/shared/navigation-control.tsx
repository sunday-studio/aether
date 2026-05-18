import { BadgeCheck, Egg } from 'lucide-react';
import { NavLink } from 'react-router';
import { cn } from '~/utils/cn';
import { RadialAvatar } from './radiant-avatar';
import { Tooltip } from './tooltip';

const NavigationControlItem = ({
	route,
}: {
	route: {
		label: string;
		route: string;
		shortcut: string[];
		icon: React.ReactNode;
	};
}) => {
	const isSettings = route.route === '/settings';

	return (
		<Tooltip
			key={route.label}
			contentClassName='text-xs'
			content={route.label}
			shortcuts={route.shortcut}
			trigger={
				<NavLink
					to={route.route}
					className={({ isActive }) =>
						cn(
							'flex h-9.5 w-9.5 shrink-0 items-center justify-center rounded-full text-sm text-(--color-navigation-control-foreground) transition-all duration-300 hover:text-(--color-navigation-control-active-foreground)',
							{
								'bg-(--color-navigation-control-active) text-(--color-navigation-control-active-foreground)':
									isActive && !isSettings,
								'hover:bg-transparent': isSettings,
							},
						)
					}
				>
					<li>{route.icon}</li>
				</NavLink>
			}
		/>
	);
};
const routes = [
	{
		label: 'Journal',
		route: '/',
		shortcut: ['⌘', 'J'],
		icon: <Egg className='size-5.5' />,
	},
	{
		label: 'Tasks',
		route: '/tasks',
		shortcut: ['⌘', 'T'],
		icon: <BadgeCheck className='size-5.5' />,
	},
	{
		label: 'Settings',
		route: '/settings',
		shortcut: ['⌘', 'S'],
		icon: <RadialAvatar size={28} seed={Math.random().toString()} />,
	},
];

export const NavigationControl = () => {
	return (
		<div className='navigation-control absolute right-1/2 bottom-5 left-1/2 z-50 w-fit -translate-x-1/2 rounded-full bg-(--color-card) p-1.5'>
			<ul className='relative flex items-center justify-center gap-1'>
				{routes.map(route => (
					<NavigationControlItem key={route.label} route={route} />
				))}
			</ul>
		</div>
	);
};
