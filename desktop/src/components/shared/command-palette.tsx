import { useQuery } from '@tanstack/react-query';
import {
	CommandDialog,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
	CommandList,
} from 'cmdk';
import {
	BadgeCheck,
	BookOpen,
	Bookmark,
	FileText,
	Goal,
	Search,
	Settings,
	Tag,
} from 'lucide-react';
import * as React from 'react';
import { useNavigate } from 'react-router';
import { customFetch } from '~/lib/api-client';

interface CommandPaletteProps {
	open: boolean;
	onOpenChange: (open: boolean) => void;
}

interface SearchResult {
	id: string;
	resourceType: string;
	resourceId: string;
	title: string;
	preview: string;
	score: number;
	matchKind: string;
	highlights: string[];
	sourceUpdatedAt: string;
	createdAt: string;
	updatedAt: string;
}

interface SearchResponse {
	results: SearchResult[];
	total: number;
	nextCursor: string | null;
	hasMore: boolean;
	query: string;
	mode: string;
}

const actions = [
	{
		label: 'Journal',
		route: '/',
		icon: <BookOpen className='size-4' />,
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

const searchableResourceTypes = new Set(['entry', 'task', 'goal', 'tag', 'bookmark']);

const resourceTypeIcon = (resourceType: string) => {
	switch (resourceType) {
		case 'entry':
			return <FileText className='size-4' />;
		case 'task':
			return <BadgeCheck className='size-4' />;
		case 'goal':
			return <Goal className='size-4' />;
		case 'tag':
			return <Tag className='size-4' />;
		case 'bookmark':
			return <Bookmark className='size-4' />;
		default:
			return <Search className='size-4' />;
	}
};

const routeForResource = (result: SearchResult) => {
	switch (result.resourceType) {
		case 'task':
			return '/tasks';
		case 'goal':
			return `/tasks/goal/${result.resourceId}`;
		case 'entry':
		case 'tag':
		case 'bookmark':
		default:
			return '/';
	}
};

const compactResourceType = (resourceType: string) => {
	if (!searchableResourceTypes.has(resourceType)) return 'result';
	return resourceType;
};

export const CommandPalette = ({ open, onOpenChange }: CommandPaletteProps) => {
	const navigate = useNavigate();
	const [searchQuery, setSearchQuery] = React.useState('');
	const [debouncedSearchQuery, setDebouncedSearchQuery] = React.useState('');

	React.useEffect(() => {
		const timeout = window.setTimeout(() => {
			setDebouncedSearchQuery(searchQuery.trim());
		}, 180);

		return () => window.clearTimeout(timeout);
	}, [searchQuery]);

	React.useEffect(() => {
		if (!open) {
			setSearchQuery('');
			setDebouncedSearchQuery('');
		}
	}, [open]);

	const searchResultsQuery = useQuery({
		queryKey: ['command-palette-search', debouncedSearchQuery],
		enabled: open && debouncedSearchQuery.length >= 2,
		queryFn: async () => {
			const response = await customFetch<{ data: SearchResponse; status: number }>(
				`/v1/search?q=${encodeURIComponent(debouncedSearchQuery)}&mode=hybrid&limit=8`,
				{ method: 'GET' },
			);

			return response.data;
		},
	});

	const handleSelect = (route: string) => {
		onOpenChange(false);
		setSearchQuery('');
		navigate(route);
	};

	const handleResourceSelect = (result: SearchResult) => {
		handleSelect(routeForResource(result));
	};

	const shouldShowSearch = searchQuery.trim().length >= 2;
	const searchResults = searchResultsQuery.data?.results ?? [];

	return (
		<CommandDialog open={open} onOpenChange={onOpenChange}>
			<CommandInput
				placeholder='Search or go to...'
				value={searchQuery}
				onValueChange={setSearchQuery}
			/>
			<CommandList>
				<CommandEmpty>No results found</CommandEmpty>
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
				{shouldShowSearch && (
					<CommandGroup heading='Search'>
						{searchResultsQuery.isFetching ? (
							<div className='command-palette-loading px-3 py-2 text-sm'>Searching...</div>
						) : null}
						{searchResults.map(result => (
							<CommandItem
								key={result.id}
								value={`${result.title} ${result.preview} ${result.resourceType}`}
								onSelect={() => handleResourceSelect(result)}
							>
								<div className='flex min-w-0 flex-1 items-start gap-3'>
									<div className='mt-0.5'>{resourceTypeIcon(result.resourceType)}</div>
									<div className='min-w-0 flex-1'>
										<div className='flex min-w-0 items-center gap-2'>
											<span className='truncate font-medium'>
												{result.title || result.resourceType}
											</span>
											<span className='command-palette-type-badge'>
												{compactResourceType(result.resourceType)}
											</span>
										</div>
										{result.preview ? (
											<p className='mt-0.5 line-clamp-2 text-xs text-neutral-500'>
												{result.preview}
											</p>
										) : null}
									</div>
								</div>
							</CommandItem>
						))}
					</CommandGroup>
				)}
			</CommandList>
		</CommandDialog>
	);
};
