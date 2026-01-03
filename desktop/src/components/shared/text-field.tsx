import {
	TextField as AriaTextField,
	type TextFieldProps as AriaTextFieldProps,
	TextArea,
	type ValidationResult,
} from "react-aria-components";
import { cn, tv } from "tailwind-variants";
import { composeTailwindRenderProps, focusRing } from "~/utils/cn";
import {
	Description,
	FieldError,
	fieldBorderStyles,
	Input,
	Label,
} from "./field";

const inputStyles = tv({
	extend: focusRing,
	base: "rounded-xl min-h-9 text-sm box-border transition bg-neutral-100 text-neutral-600",
	variants: {
		isFocused: fieldBorderStyles.variants.isFocusWithin,
		isInvalid: fieldBorderStyles.variants.isInvalid,
		isDisabled: fieldBorderStyles.variants.isDisabled,
	},
});

export interface TextFieldProps extends AriaTextFieldProps {
	label?: string;
	description?: string;
	placeholder?: string;
	errorMessage?: string;
}

export function TextField({
	label,
	description,
	errorMessage,
	...props
}: TextFieldProps) {
	return (
		<AriaTextField
			{...props}
			className={composeTailwindRenderProps(
				props.className,
				"flex flex-col gap-1",
			)}
		>
			{label && <Label>{label}</Label>}
			<Input className={inputStyles} />
			{description && <Description>{description}</Description>}
			<FieldError value={errorMessage} />
		</AriaTextField>
	);
}

export function TextAreaField({ label, ...props }: TextFieldProps) {
	return (
		<AriaTextField
			{...props}
			className={composeTailwindRenderProps(
				props.className,
				"flex flex-col gap-1",
			)}
		>
			{label && <Label>{label}</Label>}
			<TextArea
				rows={4}
				className={composeTailwindRenderProps(
					inputStyles,
					"py-2! placeholder:text-neutral-500  px-3 focus:outline-0",
				)}
			/>
		</AriaTextField>
	);
}
