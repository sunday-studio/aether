interface TaskTitleInputProps {
	value: string | undefined;
	onChange: (value: string) => void;
}

export const TaskTitleInput = ({ value, onChange }: TaskTitleInputProps) => {
	return (
		<input
			onBlur={(e) => onChange(e.target.value)}
			placeholder="Add a task"
			className="w-full text-md font-medium text-neutral-600"
			defaultValue={value}
		/>
	);
};
