import { Disc } from "lucide-react";
import { forwardRef, useMemo, useState } from "react";
import { Button, DialogTrigger, Popover } from "react-aria-components";
import { useAddGoalToTask, useGetGoals } from "~/aether-sdk";
import type { Goal, Task } from "~/aether-sdk/models";
import { Radio, RadioGroup } from "~/components/shared/radio";
import {
	popoverContentStyles,
	searchInputStyles,
} from "~/components/shared/tags-popover-selector";
import { Tooltip } from "~/components/shared/tooltip";
import { useOptimisticUpdateTaskQuery } from "../../use-optimistic-task-hooks";
import { TaskActionButton } from "./task-shared-components";

interface TaskGoalSelectorProps {
	taskId: string;
	value: Task["goalInstanceId"] | undefined;
}

const CustomTrigger = forwardRef<
	HTMLDivElement,
	{ goalName: string; hasGoal: boolean } & React.HTMLAttributes<HTMLDivElement>
>(({ goalName, hasGoal, ...rest }, ref) => {
	return (
		<TaskActionButton
			ref={ref}
			className={hasGoal ? "w-auto px-1.5 text-xs" : undefined}
			{...rest}
		>
			{!hasGoal && <Disc size={15} strokeWidth={3} className="" />}
			{goalName}
		</TaskActionButton>
	);
});

export const TaskGoalSelector = ({ taskId, value }: TaskGoalSelectorProps) => {
	const { data: goalsResponse } = useGetGoals();
	const { mutate: addGoalToTask } = useAddGoalToTask();
	const [searchValue, setSearchValue] = useState("");

	const { updateLocalInstance } = useOptimisticUpdateTaskQuery();

	// SDK now returns properly typed PaginatedGoals
	const goalsData: Goal[] = goalsResponse?.data?.items ?? [];

	const selectedGoal = useMemo(() => {
		return goalsData.find((goal) => goal.id === value);
	}, [goalsData, value]);

	const filteredGoals = useMemo(() => {
		return goalsData.filter((goal) =>
			goal.name?.toLowerCase().includes(searchValue.toLowerCase()),
		);
	}, [goalsData, searchValue]);

	const handleOnSelectGoal = (goalId: string) => {
		addGoalToTask(
			{
				id: taskId,
				data: {
					goalId,
				},
			},
			{
				onSuccess: ({ data }) => {
					updateLocalInstance({
						id: taskId,
						data: {
							goalInstanceId: data?.goalInstanceId ?? "",
						},
					});
				},
			},
		);
	};

	return (
		<DialogTrigger>
			<Button>
				<Tooltip
					trigger={
						<CustomTrigger
							goalName={selectedGoal?.name ?? ""}
							hasGoal={selectedGoal !== undefined}
						/>
					}
					content="Select goal"
					disabled={Boolean(selectedGoal)}
				/>
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
