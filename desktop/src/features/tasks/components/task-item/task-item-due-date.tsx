import { format } from "date-fns";
import { Bell } from "lucide-react";
import { DateTimePicker } from "~/components/shared/datepicker";

interface TaskDueDateInputProps {
	value: string | undefined;
	onChange: (value: string) => void;
}

export const TaskDueDateInput = ({
	value,
	onChange,
}: TaskDueDateInputProps) => {
	const trigger = value ? (
		<span className=" text-neutral-500 text-[13px] flex items-center justify-center">
			Due on {format(value, "do MMM, yyyy • p")}
		</span>
	) : (
		<span className="w-6 h-6 rounded-lg bg-neutral-200 text-neutral-400 text-sm flex items-center justify-center focus:outline-2 focus:outline-offset-1 focus:outline-neutral-300 active:bg-neutral-300 active:outline-2 active:outline-offset-1 active:outline-neutral-300">
			<Bell size={14} strokeWidth={3} className="" />
		</span>
	);

	return (
		<div className="shrink-0 w-fit">
			<DateTimePicker onChange={onChange} trigger={trigger} />
		</div>
	);
};
