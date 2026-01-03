import { useEffect, useRef, useState } from "react";
import { useDebouncedValue } from "~/hooks/use-debounce";

interface TaskDescriptionInputProps {
	value: string | undefined;
	onChange: (value: string) => void;
}

export const TaskDescriptionInput: React.FC<TaskDescriptionInputProps> = ({
	value,
	onChange,
}) => {
	const [inputValue, setInputValue] = useState(value ?? "");
	const debouncedValue = useDebouncedValue(inputValue, 500);
	const divRef = useRef<HTMLDivElement>(null);

	// Update debounced value
	useEffect(() => {
		if (debouncedValue !== value) {
			onChange(debouncedValue);
		}
	}, [debouncedValue, onChange, value]);

	// Only sync external value changes when user isn't editing
	useEffect(() => {
		const div = divRef.current;
		if (!div) return;

		// Don't update if the div is focused (user is typing)
		if (document.activeElement === div) return;

		// Only update if value actually changed
		if (value !== div.textContent) {
			div.textContent = value ?? "";
			setInputValue(value ?? "");
		}
	}, [value]);

	const handleInput = (e: React.FormEvent<HTMLDivElement>) => {
		setInputValue(e.currentTarget.textContent ?? "");
	};

	return (
		<div
			ref={divRef}
			onInput={handleInput}
			contentEditable
			suppressContentEditableWarning
			data-placeholder="Add a description"
			className="w-full text-sm text-neutral-500 font-normal outline-none whitespace-pre-wrap
             empty:before:content-[attr(data-placeholder)]
             empty:before:text-neutral-400"
		/>
	);
};
