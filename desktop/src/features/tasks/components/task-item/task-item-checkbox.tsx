import { cn } from "~/utils/cn";

interface TaskCheckboxProps {
	isChecked: boolean;
	onChange: (isChecked: boolean) => void;
}

const AnimatedCheck = () => (
	<svg
		aria-hidden="true"
		focusable="false"
		viewBox="0 0 24 24"
		className="w-3.5 h-3.5"
		fill="none"
		stroke="currentColor"
		strokeWidth={3}
		strokeLinecap="round"
		strokeLinejoin="round"
	>
		<path d="M5 13l4 4L19 7" className="check-path" />
	</svg>
);

export const TaskItemCheckbox = ({
	isChecked,
	onChange,
}: TaskCheckboxProps) => {
	return (
		<button
			onClick={() => onChange(!isChecked)}
			type="button"
			className={cn(
				" w-4.5 h-4.5 flex items-center justify-center rounded-full ring-2 ring-neutral-300 cursor-pointer transition-all duration-200 hover:ring-green-600",
				isChecked && "bg-green-100 ring-green-600",
			)}
		>
			{isChecked && (
				<span className="text-green-800">
					<AnimatedCheck />
				</span>
			)}
		</button>
	);
};
