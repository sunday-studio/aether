import { useDebounceCallback } from "~/hooks/use-debounce";

interface TaskTitleInputProps {
	value: string | undefined;
	onChange: (value: string) => void;
}

export const TaskTitleInput = ({ value, onChange }: TaskTitleInputProps) => {
	const debouncedOnChange = useDebounceCallback(onChange, 500);

	return (
		<input
			defaultValue={value}
			onChange={(e) => debouncedOnChange(e.target.value)}
			placeholder="Add a task"
			className="w-full text-sm font-inter font-medium text-neutral-700  outline-none"
		/>
	);
};
