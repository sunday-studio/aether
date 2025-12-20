import { format, isToday } from "date-fns";

export const TaskListDivider = ({ date }: { date: string }) => {
	return (
		<div className="flex items-center justify-between gap-4 my-6">
			<div className="shrink-0 bg-linear-to-b from-neutral-100 to-neutral-200 py-1.5 rounded-full px-3">
				<p className="text-neutral-600 text-xs font-medium">
					{isToday(new Date(date))
						? "Today"
						: format(new Date(date), "d MMM, yyyy")}
				</p>
			</div>
			<div className="w-full h-0.5 bg-neutral-50 rounded-full" />
		</div>
	);
};
