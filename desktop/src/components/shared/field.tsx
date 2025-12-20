import React from "react";
import {
	composeRenderProps,
	type FieldErrorProps,
	Group,
	type GroupProps,
	type InputProps,
	type LabelProps,
	FieldError as RACFieldError,
	Input as RACInput,
	Label as RACLabel,
	Text,
	type TextProps,
} from "react-aria-components";
import { twMerge } from "tailwind-merge";
import { tv } from "tailwind-variants";
import { composeTailwindRenderProps, focusRing } from "~/utils/cn";

export function Label(props: LabelProps) {
	return (
		<RACLabel
			{...props}
			className={twMerge(
				"font-sans text-sm text-neutral-600 font-medium cursor-default w-fit",
				props.className,
			)}
		/>
	);
}

export function Description(props: TextProps) {
	return (
		<Text
			{...props}
			slot="description"
			className={twMerge(
				"text-sm text-neutral-600 font-medium",
				props.className,
			)}
		/>
	);
}

export function FieldError(props: FieldErrorProps) {
	return (
		<RACFieldError
			{...props}
			className={composeTailwindRenderProps(
				props.className,
				"text-sm text-red-600 font-medium forced-colors:text-[Mark]",
			)}
		/>
	);
}

export const fieldBorderStyles = tv({
	base: "transition",
	variants: {
		isFocusWithin: {
			false:
				"border-neutral-300 hover:border-neutral-400 forced-colors:border-[ButtonBorder]",
			true: "border-neutral-600 forced-colors:border-[Highlight]",
		},
		isInvalid: {
			true: "border-red-600 forced-colors:border-[Mark]",
		},
		isDisabled: {
			true: "border-neutral-100 forced-colors:border-[GrayText]",
		},
	},
});

export const fieldGroupStyles = tv({
	extend: focusRing,
	base: "group flex items-center h-9 box-border bg-neutral-100 forced-colors:bg-[Field] border rounded-lg overflow-hidden transition",
	variants: fieldBorderStyles.variants,
});

export function FieldGroup(props: GroupProps) {
	return (
		<Group
			{...props}
			className={composeRenderProps(props.className, (className, renderProps) =>
				fieldGroupStyles({ ...renderProps, className }),
			)}
		/>
	);
}

export function Input(props: InputProps) {
	return (
		<RACInput
			{...props}
			className={composeTailwindRenderProps(
				props.className,
				"px-3 py-0 min-h-9 flex-1 min-w-0 border-0 outline outline-0 bg-neutral-100 font-sans text-sm font-medium text-neutral-800 placeholder:text-neutral-600 disabled:text-neutral-100 disabled:placeholder:text-neutral-100 [-webkit-tap-highlight-color:transparent]",
			)}
		/>
	);
}
