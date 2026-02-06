import { format, isToday } from "date-fns";
import { cn } from "~/utils/cn";

interface TaskListDividerProps {
	date: string | undefined;
	isOverdue?: boolean;
	completedCountLabel?: string;
	title?: string;
}
export const TaskListDivider = ({
	date,
	isOverdue,
	completedCountLabel,
	title,
}: TaskListDividerProps) => {
	const label = isOverdue
		? "Overdue"
		: date && isToday(new Date(date))
			? "Today"
			: date && format(new Date(date), "d MMM, yyyy");

	return (
		<div className={cn("flex items-center justify-between gap-4 my-6")}>
			<div
				className={cn(
					"shrink-0 bg-linear-to-b from-(--color-tasklist-label-start) to-(--color-tasklist-label-end) text-neutral-600  py-1 rounded-lg px-1.5 ring ring-(--color-divider)",
					{
						"ring-rose-200 from-rose-100 to-rose-200 text-rose-700": isOverdue,
					},
				)}
			>
				<p className=" text-xs select-none">{title ?? label}</p>
			</div>
			<div className="w-full h-0.5 bg-(--color-divider) rounded-full" />
			{completedCountLabel && (
				<p className="text-xs shrink-0 font-medium text-neutral-500">
					{completedCountLabel}
				</p>
			)}
		</div>
	);
};
