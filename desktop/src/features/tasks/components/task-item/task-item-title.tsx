import { useEffect, useState } from "react";
import { useDebouncedValue } from "~/hooks/use-debounce";

interface TaskTitleInputProps {
	value: string | undefined;
	onChange: (value: string) => void;
}

export const TaskTitleInput = ({ value, onChange }: TaskTitleInputProps) => {
	const [inputValue, setInputValue] = useState(value ?? "");

	const debouncedValue = useDebouncedValue(inputValue, 500);

	useEffect(() => {
		if (debouncedValue !== value) {
			// onChange(debouncedValue);
		}
	}, [debouncedValue, onChange, value]);

	return (
		<input
			value={inputValue}
			onChange={(e) => setInputValue(e.target.value)}
			placeholder="Add a task"
			className="w-full text-sm font-medium text-neutral-600 outline-none"
		/>
	);
};
