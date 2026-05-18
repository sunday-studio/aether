import { CommandDialog, CommandGroup, CommandInput, CommandItem, CommandList } from 'cmdk';
import { BadgeCheck, Egg, Settings } from 'lucide-react';
import * as React from 'react';
import { useNavigate } from 'react-router';

interface CommandPaletteProps {
	open: boolean;
	onOpenChange: (open: boolean) => void;
}

const actions = [
	{
		label: 'Journal',
		route: '/',
		icon: <Egg className='size-4' />,
	},
	{
		label: 'Tasks',
		route: '/tasks',
		icon: <BadgeCheck className='size-4' />,
	},
	{
		label: 'Settings',
		route: '/settings',
		icon: <Settings className='size-4' />,
	},
];

export const CommandPalette = ({ open, onOpenChange }: CommandPaletteProps) => {
	const navigate = useNavigate();
	const [searchQuery, setSearchQuery] = React.useState('');

	const handleSelect = (route: string) => {
		onOpenChange(false);
		setSearchQuery('');
		navigate(route);
	};

	return (
		<CommandDialog open={open} onOpenChange={onOpenChange}>
			<CommandInput placeholder='Go to...' value={searchQuery} onValueChange={setSearchQuery} />
			<CommandList>
				<CommandGroup heading='Navigation'>
					{actions.map(action => (
						<CommandItem
							key={action.route}
							value={action.label}
							onSelect={() => handleSelect(action.route)}
						>
							<div className='flex w-full items-center gap-2'>
								{action.icon}
								<span>{action.label}</span>
							</div>
						</CommandItem>
					))}
				</CommandGroup>
			</CommandList>
		</CommandDialog>
	);
};
