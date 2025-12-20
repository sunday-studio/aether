import { Link } from "react-router";
import { useGetGoals } from "~/aether-sdk";
import { CreateGoalDialog } from "./goals/create-goal-dialog";

const NavigationItem = ({ goal, route }: { goal: string; route: string }) => {
	return (
		<Link to={route}>
			<li className="text-xs text-neutral-600 p-1 px-1.5 cursor-pointer rounded-md hover:bg-neutral-200 inline-flex">
				{goal}
			</li>
		</Link>
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
				<NavigationItem route="/tasks/overdue" goal="Overdue" />
			</div>
			<GoalsList />
		</div>
	);
};
