/** biome-ignore-all lint/a11y/useSemanticElements: <explanation> */
import { parseDate } from "@internationalized/date";
import { format } from "date-fns";
import { Bell, X } from "lucide-react";
import { DateTimePicker } from "~/components/shared/datepicker";

interface TaskDueDateInputProps {
	value: string | undefined;
	onChange: (value: string | null) => void;
}

const SQLITE_NULL_DATE = "0001-01-01T00:00:00Z";

const getDateValue = (value: string | undefined) => {
	if (SQLITE_NULL_DATE === value || !value) {
		return undefined;
	}

	const dateonly = value?.split("T")[0];

	return parseDate(dateonly as string);
};

export const TaskDueDateInput = ({
	value,
	onChange,
}: TaskDueDateInputProps) => {
	const trigger = value ? (
		<p className="h-6 flex items-center justify-between px-2 transition-colors rounded-lg group gap-1 -ml-2 hover:bg-neutral-200">
			<span className="text-xs text-neutral-500  block">
				Due on {format(value, "do MMM, yyyy")}
			</span>

			<span
				role="button"
				className="text-xs text-transparent group-hover:text-neutral-400 cursor-pointer hover:bg-neutral-300 w-3 h-3 rounded-sm flex items-center justify-center"
				onClick={(e) => {
					e.preventDefault();
					e.stopPropagation();
					onChange(null);
				}}
				aria-label="Clear due date"
				tabIndex={0}
				onKeyDown={(e) => {
					if (e.key === "Enter" || e.key === " ") {
						e.preventDefault();
						e.stopPropagation();
						onChange(null);
					}
				}}
			>
				<X size={12} />
			</span>
		</p>
	) : (
		<span className="w-6 h-6 rounded-lg bg-neutral-200 text-neutral-400 text-sm flex items-center justify-center focus:outline-2 focus:outline-offset-1 focus:outline-neutral-300 active:bg-neutral-300 active:outline-2 active:outline-offset-1 active:outline-neutral-300">
			<Bell size={14} strokeWidth={3} className="" />
		</span>
	);

	return (
		<div className="shrink-0 h-6">
			<DateTimePicker
				value={getDateValue(value)}
				onChange={onChange}
				trigger={trigger}
			/>
		</div>
	);
};
