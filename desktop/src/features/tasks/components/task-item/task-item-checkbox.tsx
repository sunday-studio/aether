import { cn } from "~/utils/cn";

interface TaskCheckboxProps {
	isChecked: boolean;
	onChange: (isChecked: boolean) => void;
}

export const TaskItemCheckbox = ({
	isChecked,
	onChange,
}: TaskCheckboxProps) => {
	return (
		<button
			onClick={() => onChange(!isChecked)}
			type="button"
			className={cn(
				`
        w-4 h-4 flex items-center justify-center rounded-full
        cursor-pointer transition-all duration-200 ease-out
        ring-2 ring-neutral-300 hover:ring-green-600
        focus:outline-2 focus:outline-offset-2 focus:outline-green-600
        `,
				isChecked && "bg-green-100 ring-green-600",
			)}
		/>
	);
};
