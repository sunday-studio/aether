import { Disc } from "lucide-react";
import { forwardRef, useMemo, useState } from "react";
import { Button, DialogTrigger, Popover } from "react-aria-components";
import { useAddGoalToTask, useGetGoals } from "~/aether-sdk";
import type { DbTask } from "~/aether-sdk/models";
import { Radio, RadioGroup } from "~/components/shared/radio";
import {
	popoverContentStyles,
	searchInputStyles,
} from "~/components/shared/tags-popover-selector";

interface TaskGoalSelectorProps {
	taskId: string;
	value: DbTask["goalInstanceId"] | undefined;
}

const CustomTrigger = forwardRef<
	HTMLDivElement,
	{ goalName: string } & React.HTMLAttributes<HTMLDivElement>
>(({ goalName, ...rest }, ref) => {
	console.log({ goalName });
	return (
		<div ref={ref} {...rest}>
			<Disc size={15} strokeWidth={3} className="-mt-0.5" />
			{goalName}
		</div>
	);
});

export const TaskGoalSelector = ({ taskId, value }: TaskGoalSelectorProps) => {
	const { data: goals } = useGetGoals();
	const { mutate: addGoalToTask } = useAddGoalToTask();
	const [searchValue, setSearchValue] = useState("");

	const selectedGoal = useMemo(() => {
		return goals?.data.find((goal) => goal.id === value);
	}, [goals, value]);

	const filteredGoals = useMemo(() => {
		return goals?.data.filter((goal) =>
			goal.name?.toLowerCase().includes(searchValue.toLowerCase()),
		);
	}, [goals, searchValue]);

	const handleOnSelectGoal = (goalId: string) => {
		addGoalToTask(
			{
				id: taskId,
				data: {
					goalId,
				},
			},
			{
				onSuccess: () => {
					// queryClient.invalidateQueries({ queryKey: tasksQueryKey });
				},
			},
		);
	};

	return (
		<DialogTrigger>
			<Button>
				<CustomTrigger goalName={selectedGoal?.name ?? ""} />
			</Button>
			<Popover className={popoverContentStyles}>
				<div className="sticky top-0 pb-1">
					<input
						type="text"
						placeholder="Search goals..."
						value={searchValue}
						onChange={(e) => setSearchValue(e.target.value)}
						className={searchInputStyles}
					/>
				</div>
				<div className="max-h-48 overflow-y-auto">
					<RadioGroup
						value={selectedGoal?.id ?? ""}
						onChange={(value) => handleOnSelectGoal(value as string)}
					>
						{filteredGoals?.map((goal) => (
							<Radio key={goal.id} value={goal.id ?? ""}>
								{goal.name}
							</Radio>
						))}
					</RadioGroup>
				</div>
			</Popover>
		</DialogTrigger>
	);
};
