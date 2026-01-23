import type { ReactNode } from "react";
import {
	composeRenderProps,
	Radio as RACRadio,
	RadioGroup as RACRadioGroup,
	type RadioGroupProps as RACRadioGroupProps,
	type RadioProps,
} from "react-aria-components";
import { tv } from "tailwind-variants";
import { composeTailwindRenderProps, focusRing } from "~/utils/cn";
import { Description, FieldError, Label } from "./field";
import { popoverItemStyles } from "./tags-popover-selector";

export interface RadioGroupProps extends Omit<RACRadioGroupProps, "children"> {
	label?: string;
	children?: ReactNode;
	description?: string;
	errorMessage?: string;
}

export function RadioGroup(props: RadioGroupProps) {
	return (
		<RACRadioGroup
			{...props}
			className={composeTailwindRenderProps(
				props.className,
				"group flex flex-col gap-1",
			)}
		>
			{props.label && <Label>{props.label}</Label>}
			<div className="flex flex-col gap-2">{props.children}</div>
			{props.description && <Description>{props.description}</Description>}
			<FieldError value={props.errorMessage} />
		</RACRadioGroup>
	);
}

const styles = tv({
	extend: focusRing,
	base: "w-4 h-4 bg-neutral-600 box-border rounded-full transition-all",
	variants: {
		isSelected: {
			true: "border-[calc(var(--spacing)*1.2)] border-green-700 bg-white forced-colors:border-[Highlight]! group-pressed:border-neutral-800",
			false: "bg-linear-to-b inset-shadow-xs group-pressed:border-neutral-500",
		},
		isInvalid: {
			true: "border-red-700 group-pressed:border-red-800 forced-colors:border-[Mark]!",
		},
		isDisabled: {
			true: "border-neutral-200 forced-colors:border-[GrayText]!",
		},
	},
});

export function Radio(props: RadioProps) {
	return (
		<RACRadio
			{...props}
			className={composeTailwindRenderProps(props.className, popoverItemStyles)}
		>
			{composeRenderProps(props.children, (children, renderProps) => (
				<>
					{children}
					<div className={styles(renderProps)} />
				</>
			))}
		</RACRadio>
	);
}
