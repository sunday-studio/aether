/** biome-ignore-all lint/a11y/useSemanticElements: false positive */
/** biome-ignore-all lint/a11y/useKeyWithClickEvents: <explanation> */
import { parseDate } from "@internationalized/date";
import { format } from "date-fns";
import { Bell, X } from "lucide-react";
import { DateTimePicker } from "~/components/shared/datepicker";
import { cn } from "~/utils/cn";

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
		<p
			className={cn(
				"text-xs",
				"h-6",
				"flex items-center justify-between",
				"pr-3 pl-2",
				"transition-colors",
				"rounded-lg group gap-1",
				"bg-linear-to-b from-neutral-100 to-neutral-200",
				"inset-ring-1 inset-ring-neutral-200",
				"relative text-neutral-400",
			)}
		>
			<span className={cn("p-0 leading-none", "text-shadow-3xs")}>
				Due on {format(value, "do MMM, yyyy")}
			</span>
			<span
				role="button"
				className={cn(
					"opacity-0 group-hover:opacity-100",
					"ring-2 ring-neutral-50",
					"text-xs text-transparent group-hover:text-neutral-400",
					"cursor-pointer",
					"bg-neutral-300",
					"w-3 h-3",
					"rounded-sm flex items-center justify-center",
					"absolute -right-1 -top-1",
				)}
				onClick={(e) => {
					e.preventDefault();
					e.stopPropagation();
					onChange(null);
				}}
				aria-label="Clear due date"
				tabIndex={0}
			>
				<X size={12} />
			</span>
		</p>
	) : (
		<span
			className={cn(
				"w-6 h-6 rounded-lg",
				"bg-neutral-200 text-neutral-400 text-sm",
				"flex items-center justify-center",
				"focus:outline-2 focus:outline-offset-1 focus:outline-neutral-300",
				"active:bg-neutral-300 active:outline-2 active:outline-offset-1 active:outline-neutral-300",
			)}
		>
			<Bell size={14} strokeWidth={3} />
		</span>
	);

	return (
		<div className={cn("shrink-0")}>
			<DateTimePicker
				value={getDateValue(value)}
				onChange={onChange}
				trigger={trigger}
			/>
		</div>
	);
};
