import { useEffect, useRef } from "react";
import { useDebounceCallback } from "~/hooks/use-debounce";

interface TaskDescriptionInputProps {
	value: string | null;
	onChange: (value: string) => void;
}

export const TaskDescriptionInput: React.FC<TaskDescriptionInputProps> = ({
	value,
	onChange,
}) => {
	const divRef = useRef<HTMLDivElement>(null);
	const debouncedOnChange = useDebounceCallback(onChange, 500);

	// Set initial value and sync external changes when not focused
	useEffect(() => {
		const div = divRef.current;
		if (!div) return;

		// Don't update if the div is focused (user is typing)
		if (document.activeElement === div) return;

		// Only update if value actually changed
		if (value !== div.textContent) {
			div.textContent = value ?? "";
		}
	}, [value]);

	const handleInput = (e: React.FormEvent<HTMLDivElement>) => {
		debouncedOnChange(e.currentTarget.textContent ?? "");
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
