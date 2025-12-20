import { isBefore, startOfDay } from "date-fns";
import type { DbTask } from "~/aether-sdk/models";
import { cn } from "~/utils/cn";
import { TaskListDivider } from "../task-list-divider";
import { TaskItem } from "./task-item";

interface TasksContainerProps {
	date: string;
	tasks: DbTask[];
}

export const TasksContainer = ({ date, tasks }: TasksContainerProps) => {
	const isPastDate = isBefore(
		startOfDay(new Date(date)),
		startOfDay(new Date()),
	);

	const completedCount = tasks.reduce((count, task) => {
		return task.isCompleted ? count + 1 : count;
	}, 0);

	return (
		<li
			key={date}
			className={cn(
				"space-y-4 px-0.5",
				isPastDate && "opacity-60 hover:opacity-100",
			)}
		>
			<TaskListDivider
				date={date}
				completedCountLabel={`${completedCount} / ${tasks.length}`}
			/>
			{tasks.map((task) => (
				<TaskItem key={task.id} task={task} />
			))}
		</li>
	);
};
