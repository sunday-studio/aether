import { useEffect, useRef, useState } from "react";
import { useDebounceCallback } from "~/hooks/use-debounce";

interface TaskTitleInputProps {
	value: string | undefined;
	onChange: (value: string) => void;
}

export const TaskTitleInput = ({ value, onChange }: TaskTitleInputProps) => {
	const [inputValue, setInputValue] = useState(value ?? "");
	const inputRef = useRef<HTMLInputElement>(null);
	const debouncedOnChange = useDebounceCallback(onChange, 500);

	// Sync with external value changes when input is not focused
	useEffect(() => {
		const input = inputRef.current;
		if (!input) return;

		// Don't update if the input is focused (user is typing)
		if (document.activeElement === input) return;

		// Only update if value actually changed
		if (value !== inputValue) {
			setInputValue(value ?? "");
		}
	}, [value, inputValue]);

	const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
		const newValue = e.target.value;
		setInputValue(newValue);
		debouncedOnChange(newValue);
	};

	return (
		<input
			ref={inputRef}
			value={inputValue}
			onChange={handleChange}
			placeholder="Add a task"
			className="w-full text-sm font-inter font-medium text-neutral-600  outline-none"
		/>
	);
};
