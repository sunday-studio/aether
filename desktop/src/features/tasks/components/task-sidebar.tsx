import { NavLink } from "react-router";
import { cn } from "tailwind-variants";
import { useGetGoals } from "~/aether-sdk";
import { CreateGoalDialog } from "./goals/create-goal-dialog";

const NavigationItem = ({ goal, route }: { goal: string; route: string }) => {
	return (
		<NavLink
			to={route}
			end
			className={({ isActive }) => {
				return cn(
					"group text-xs leading-none py-1.5 px-1.5 hover:bg-neutral-200  rounded-md",
					{
						"bg-neutral-200 hover:bg-neutral-300 text-neutral-950": isActive,
					},
				);
			}}
		>
			{/* <li className="text-xs p-1 px-1.5 text-neutral-600 cursor-pointer rounded-md hover:bg-neutral-200 inline-flex w-full"> */}
			{goal}
			{/* </li> */}
		</NavLink>
	);
};

const GoalsList = () => {
	const { data: goals } = useGetGoals();

	return (
		<div className="w-full">
			<div className="px-1.5 py-2 flex items-center justify-between">
				<p className="text-sm text-neutral-800 font-medium">Goals</p>
				<CreateGoalDialog />
			</div>
			<ul className="flex flex-col gap-1 items-start">
				{goals?.data.map((goal) => (
					<NavigationItem
						key={goal.id}
						route={`/tasks/goal/${goal.id}`}
						goal={goal.name ?? ""}
					/>
				))}
			</ul>
		</div>
	);
};

export const TaskSidebar = () => {
	return (
		<div className="flex flex-col gap-4 justify-start items-start pr-5  mt-5">
			<div className="flex flex-col gap-1 items-start">
				<NavigationItem route="/tasks" goal="Inbox" />
				{/* <NavigationItem route="/tasks/overdue" goal="Overdue" /> */}
			</div>
			<GoalsList />
		</div>
	);
};
