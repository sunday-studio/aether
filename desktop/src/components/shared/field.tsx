import {
	composeRenderProps,
	Group,
	type GroupProps,
	type InputProps,
	type LabelProps,
	Input as RACInput,
	Label as RACLabel,
	Text,
	type TextProps,
} from "react-aria-components";
import { twMerge } from "tailwind-merge";
import { cn, tv } from "tailwind-variants";
import { composeTailwindRenderProps, focusRing } from "~/utils/cn";

export function Label(props: LabelProps) {
	return (
		<RACLabel
			{...props}
			className={twMerge(
				"text-sm text-neutral-500  cursor-default w-fit pl-2",
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

export function FieldError({ value }: { value?: string }) {
	if (!value) return null;
	return <span className={cn("text-sm text-rose-600 pl-2")}>{value}</span>;
}

export const fieldBorderStyles = tv({
	base: "transition",
	variants: {
		isFocusWithin: {
			false: "ring-neutral-300 hover:ring-neutral-400 ",
			true: "ring-neutral-600 ring-2 ring-neutral-200 ring-offset-1",
		},
		isInvalid: {
			true: "ring-rose-600 ring-2 ring-rose-600/20 ring-offset-1",
		},
		isDisabled: {
			true: "ring-neutral-100 ring-2 opacity-80 text-neutral-400 ring-neutral-100/20 ring-offset-1 hover:ring-transparent",
		},
	},
});

export const fieldGroupStyles = tv({
	extend: focusRing,
	base: "group flex items-center h-9 box-border bg-neutral-100 border rounded-xl transition",
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
				[
					"px-3 py-0 min-h-9 flex-1 min-w-0",
					"border-0 outline-0",
					"bg-neutral-100 text-sm",
					"placeholder:text-neutral-500",
				].join(" "),
			)}
		/>
	);
}
