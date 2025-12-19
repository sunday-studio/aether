import { Check } from "lucide-react";
import { useState } from "react";
import { cn } from "~/utils/cn";

interface TaskCheckboxProps {
	isChecked: boolean;
	onChange: (isChecked: boolean) => void;
}

export const TaskItemCheckbox = ({
	// isChecked,
	onChange,
}: TaskCheckboxProps) => {
	const [isCheckedState, setIsCheckedState] = useState(false);

	return (
		<button
			onClick={() => setIsCheckedState(!isCheckedState)}
			type="button"
			className={cn(
				" w-4 h-4 flex items-center justify-center rounded-full ring-2 ring-neutral-300 cursor-pointer transition-all duration-200 hover:ring-green-600",
				isCheckedState && "bg-green-100 ring-green-600",
				// inset-ring-2 ring-green-600 inset-ring-green-600  bg-green-800
			)}
		>
			{isCheckedState && (
				<Check className="w-3 h-3 text-green-800" strokeWidth={3} />
			)}
		</button>
	);
};
