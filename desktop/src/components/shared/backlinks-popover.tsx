import { useQuery } from '@tanstack/react-query';
import { FileText, Goal, Link2, Square } from 'lucide-react';
import { useNavigate } from 'react-router';
import { getBacklinks } from '~/aether-sdk';
import { Popover, PopoverContent, PopoverTrigger } from './popover';

const hiddenResourceTypes = new Set(['canvas', 'bookmark']);

const getIcon = (resourceType: string) => {
	switch (resourceType) {
		case 'entry':
			return <FileText size={14} />;
		case 'task':
			return <Square size={14} />;
		case 'goal':
			return <Goal size={14} />;
		case 'canvas':
		case 'bookmark':
			return <Link2 size={14} />;
		default:
			return <Link2 size={14} />;
	}
};

interface BacklinksPopoverProps {
	targetType: string;
	targetId: string;
	children: React.ReactNode;
}

export function BacklinksPopover({ targetType, targetId, children }: BacklinksPopoverProps) {
	const navigate = useNavigate();

	const { data: backlinks, isLoading } = useQuery({
		queryKey: ['backlinks', targetType, targetId],
		queryFn: async () => {
			return getBacklinks({
				targetType,
				targetId,
			});
		},
	});

	const handleNavigate = (sourceType: string, sourceId: string) => {
		switch (sourceType) {
			case 'entry':
				navigate('/');
				break;
			case 'task':
				navigate('/tasks');
				break;
			case 'goal':
				navigate(`/tasks/goal/${sourceId}`);
				break;
			case 'canvas':
			case 'bookmark':
				break;
			default:
				break;
		}
	};

	const visibleBacklinks = backlinks?.data?.filter(
		backlink => !hiddenResourceTypes.has(backlink.link.sourceType),
	);

	return (
		<Popover>
			<PopoverTrigger asChild>{children}</PopoverTrigger>
			<PopoverContent className='w-80 p-0' align='start'>
				<div className='border-b border-stone-200 p-3 dark:border-stone-700'>
					<h3 className='text-sm font-semibold text-stone-900 dark:text-stone-100'>Backlinks</h3>
					<p className='mt-0.5 text-xs text-stone-500 dark:text-stone-400'>
						Resources linking to this
					</p>
				</div>
				<div className='max-h-[300px] overflow-y-auto'>
					{isLoading ? (
						<div className='p-4 text-center text-sm text-stone-500'>Loading...</div>
					) : !visibleBacklinks || visibleBacklinks.length === 0 ? (
						<div className='p-4 text-center text-sm text-stone-500'>No backlinks found</div>
					) : (
						<ul className='divide-y divide-stone-200 dark:divide-stone-700'>
							{visibleBacklinks.map(backlink => (
								<li
									key={backlink.link.id}
									className='cursor-pointer p-3 transition-colors hover:bg-stone-50 dark:hover:bg-stone-800'
									onClick={() => handleNavigate(backlink.link.sourceType, backlink.link.sourceId)}
								>
									<div className='flex items-start gap-2'>
										<div className='mt-0.5 flex-shrink-0 text-stone-400 dark:text-stone-500'>
											{getIcon(backlink.link.sourceType)}
										</div>
										<div className='min-w-0 flex-1'>
											<p className='truncate text-sm font-medium text-stone-900 dark:text-stone-100'>
												{backlink.sourceTitle ||
													`${backlink.link.sourceType}:${backlink.link.sourceId}`}
											</p>
											{backlink.link.linkText && (
												<p className='mt-0.5 truncate text-xs text-stone-500 dark:text-stone-400'>
													{backlink.link.linkText}
												</p>
											)}
										</div>
									</div>
								</li>
							))}
						</ul>
					)}
				</div>
			</PopoverContent>
		</Popover>
	);
}
