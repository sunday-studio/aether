/** biome-ignore-all lint/a11y/noStaticElementInteractions: <explanation> */
interface TaskDescriptionInputProps {
	value: string | undefined;
	onChange: (value: string) => void;
}

export const TaskDescriptionInput: React.FC<TaskDescriptionInputProps> = ({
	value,
	onChange,
}) => {
	return (
		<div
			contentEditable
			suppressContentEditableWarning
			data-placeholder="Add a description"
			className="w-full text-sm text-neutral-500 font-normal outline-none whitespace-pre-wrap
             empty:before:content-[attr(data-placeholder)]
             empty:before:text-neutral-400"
			onBlur={(e) => onChange(e.currentTarget.textContent ?? "")}
		>
			{value}
		</div>
	);
};
