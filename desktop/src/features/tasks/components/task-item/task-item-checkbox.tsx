import { Badge, BadgeCheck } from "lucide-react";

interface TaskCheckboxProps {
	isChecked: boolean;
	onChange: (isChecked: boolean) => void;
}

export const TaskItemCheckbox = ({
	isChecked,
	onChange,
}: TaskCheckboxProps) => {
	return (
		<button onClick={() => onChange(!isChecked)} type="button">
			{isChecked ? (
				<BadgeCheck size={20} strokeWidth={3} className="text-green-600" />
			) : (
				<Badge size={20} strokeWidth={3} className="text-neutral-400" />
			)}
		</button>
	);
};
