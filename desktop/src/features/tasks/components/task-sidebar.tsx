import { Plus } from 'lucide-react';
import { NavLink } from 'react-router';
import { cn } from 'tailwind-variants';
import { useGetGoals } from '~/aether-sdk';
import type { Goal } from '~/aether-sdk/models';
import { Tooltip } from '~/components/shared/tooltip';
import { GoalFormDialog } from './goals/goal-form-dialog';
import { TaskActionButton } from './task-item/task-shared-components';

const NavigationItem = ({ label, route }: { label: string; route: string }) => {
	return (
		<NavLink
			to={route}
			end
			className={({ isActive }) => {
				// return cn(
				// 	'group relative -mx-1.5 rounded-md px-1.5 py-1 text-xs leading-[12px] text-(--color-secondary-text) hover:text-(--color-secondary-text-hover)',
				// 	{
				// 		'text-(--color-active-text) before:absolute before:top-1/2 before:left-[-10px] before:block before:h-2 before:w-2 before:-translate-y-1/2 before:-skew-y-3 before:rounded-full before:bg-(--color-active-text)':
				// 			isActive,
				// 	},
				// );

				return cn(
					'flex h-8 cursor-pointer items-center gap-2 rounded-full px-2.5 text-xs text-(--color-secondary-text) hover:text-(--color-secondary-text-hover)',
					{
						'bg-(--color-navigation-control-active) text-(--color-navigation-control-active-foreground) hover:text-(--color-navigation-control-active-foreground)':
							isActive,
						'bg-neutral-100 hover:text-(--color-navigation-control-foreground)': !isActive,
					},
				);
			}}
		>
			{label}
		</NavLink>
	);
};

const GoalsList = () => {
	const { data: goalsResponse } = useGetGoals();

	const goals: Goal[] = goalsResponse?.data?.items ?? [];

	return (
		<div className='w-full'>
			<div className='flex items-center justify-between gap-2 py-2'>
				<p className='text-sm font-medium text-(--color-primary-text)'>Goals</p>
				<div className='h-px w-full bg-(--color-divider)'></div>
				<GoalFormDialog
					trigger={
						<Tooltip
							trigger={
								<TaskActionButton className='cursor-pointer bg-transparent hover:bg-neutral-200'>
									<Plus size={14} strokeWidth={3} />
								</TaskActionButton>
							}
							content='Create a new goal'
							shortcuts={['⌘', 'G']}
						/>
					}
				/>
			</div>
			<ul className='flex flex-col items-start gap-1'>
				{goals.map(goal => (
					<NavigationItem key={goal.id} route={`/tasks/goal/${goal.id}`} label={goal.name ?? ''} />
				))}
			</ul>
		</div>
	);
};

export const TaskSidebar = () => {
	return (
		<div className='mt-5 flex flex-col items-start justify-start gap-4 pr-5'>
			<div className='flex items-start gap-1'>
				<NavigationItem route='/tasks' label='Inbox' />
				<NavigationItem route='/tasks/overdue' label='Overdue' />
				<NavigationItem route='/tasks/overdue' label='Overdue' />
			</div>
			{/* <GoalsList /> */}
		</div>
	);
};
