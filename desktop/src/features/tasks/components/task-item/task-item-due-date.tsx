import { format } from "date-fns";
import { CalendarDays } from "lucide-react";

interface TaskDueDateInputProps {
	value: string | undefined;
	onChange: (value: string) => void;
}

export const TaskDueDateInput = ({
	value,
	onChange,
}: TaskDueDateInputProps) => {
	if (!value)
		return (
			<p className="w-6 h-6 rounded-lg bg-neutral-100 text-neutral-500 text-sm">
				{/* <CalendarDays className="w-3 h-3" strokeWidth={2} /> */}
			</p>
		);
	return (
		<input
			type="date"
			className="w-40 text-sm"
			value={value ? format(value, "yyyy-MM-dd") : ""}
			onChange={(e) => onChange(e.target.value)}
		/>
	);
};
