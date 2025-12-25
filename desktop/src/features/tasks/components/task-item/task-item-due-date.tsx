/** biome-ignore-all lint/a11y/useSemanticElements: false positive */
/** biome-ignore-all lint/a11y/useKeyWithClickEvents: false positive */
import { type CalendarDate, parseDate } from "@internationalized/date";
import { format } from "date-fns";
import { Bell, X } from "lucide-react";
import { useMemo } from "react";
import { DateTimePicker } from "~/components/shared/datepicker";
import { Tooltip } from "~/components/shared/tooltip";
import { cn } from "~/utils/cn";
import { TaskActionButton } from "./task-shared-components";

interface TaskDueDateInputProps {
	value: string | undefined;
	onChange: (value: CalendarDate | null) => void;
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
	const trigger = useMemo(() => {
		return value ? (
			<p
				className={cn(
					"text-xs",
					"h-6",
					"flex items-center justify-between",
					"pr-3 pl-2",
					"transition-colors",
					"rounded-lg group gap-1",
					"bg-neutral-200/70",
					"relative text-neutral-500",
				)}
			>
				<span className={cn("p-0 leading-none")}>
					Due on {format(value, "do MMM, yyyy")}
				</span>
				<span
					role="button"
					className={cn(
						"opacity-0 group-hover:opacity-100 ",
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
			<TaskActionButton>
				<Bell size={14} strokeWidth={3} />
			</TaskActionButton>
		);
	}, [value, onChange]);

	return (
		<div className={cn("shrink-0")}>
			<DateTimePicker
				value={getDateValue(value)}
				onChange={onChange}
				trigger={
					<Tooltip
						content="Set due date"
						trigger={trigger}
						disabled={Boolean(value)}
						// disabled={value !== undefined}
					/>
				}
			/>
		</div>
	);
};
