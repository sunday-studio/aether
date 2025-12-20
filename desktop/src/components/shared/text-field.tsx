import {
	TextField as AriaTextField,
	type TextFieldProps as AriaTextFieldProps,
	type ValidationResult,
} from "react-aria-components";
import { tv } from "tailwind-variants";
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
	base: "border-1 rounded-lg min-h-9 font-sans text-sm py-0 px-3 box-border transition",
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
	errorMessage?: string | ((validation: ValidationResult) => string);
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
				"flex flex-col gap-1 font-sans",
			)}
		>
			{label && <Label>{label}</Label>}
			<Input className={inputStyles} />
			{description && <Description>{description}</Description>}
			<FieldError>{errorMessage}</FieldError>
		</AriaTextField>
	);
}

export function TextAreaField({
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
				"flex flex-col gap-1 font-sans",
			)}
		>
			{label && <Label>{label}</Label>}
			<Input className={inputStyles} />
		</AriaTextField>
	);
}
