import { Plus } from "lucide-react";
import { NavLink } from "react-router";
import { cn } from "tailwind-variants";
import { useGetGoals } from "~/aether-sdk";
import type { Goal } from "~/aether-sdk/models";
import { Tooltip } from "~/components/shared/tooltip";
import { GoalFormDialog } from "./goals/goal-form-dialog";
import { TaskActionButton } from "./task-item/task-shared-components";

const NavigationItem = ({ label, route }: { label: string; route: string }) => {
	return (
		<NavLink
			to={route}
			end
			className={({ isActive }) => {
				return cn(
					"group relative text-xs py-1 leading-[12px] px-1.5 -mx-1.5 rounded-md hover:bg-neutral-100",
					{
						"before:absolute before:top-1/2 before:left-[-10px] before:-translate-y-1/2 before:block before:-skew-y-3 before:h-2 before:w-2 before:rounded-full before:bg-green-700 text-green-900":
							isActive,
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

	// SDK now returns properly typed PaginatedGoals
	const goals: Goal[] = goalsResponse?.data?.items ?? [];

	return (
		<div className="w-full">
			<div className=" py-2 flex items-center justify-between gap-2">
				<p className="text-sm text-neutral-800 font-medium">Goals</p>
				<div className="w-full h-px bg-neutral-200/50"></div>
				<GoalFormDialog
					trigger={
						<Tooltip
							trigger={
								<TaskActionButton className="bg-transparent hover:bg-neutral-200 cursor-pointer">
									<Plus size={14} strokeWidth={3} />
								</TaskActionButton>
							}
							content="Create a new goal"
							shortcuts={["⌘", "G"]}
						/>
					}
				/>
			</div>
			<ul className="flex flex-col gap-1 items-start">
				{goals.map((goal) => (
					<NavigationItem
						key={goal.id}
						route={`/tasks/goal/${goal.id}`}
						label={goal.name ?? ""}
					/>
				))}
			</ul>
		</div>
	);
};

export const TaskSidebar = () => {
	return (
		<div className="flex flex-col gap-4 justify-start items-start pr-5 mt-5">
			<div className="flex flex-col gap-1 items-start">
				<NavigationItem route="/tasks" label="Inbox" />
				<NavigationItem route="/tasks/overdue" label="Overdue" />
			</div>
			<GoalsList />
		</div>
	);
};
