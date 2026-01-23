"use client";
import { Check } from "lucide-react";
import {
	ListBox as AriaListBox,
	ListBoxItem as AriaListBoxItem,
	type ListBoxProps as AriaListBoxProps,
	Collection,
	composeRenderProps,
	Header,
	type ListBoxItemProps,
	ListBoxSection,
	type SectionProps,
} from "react-aria-components";
import { tv } from "tailwind-variants";
import { cn, composeTailwindRenderProps } from "~/utils/cn";

interface ListBoxProps<T>
	extends Omit<AriaListBoxProps<T>, "layout" | "orientation"> {}

export function ListBox<T extends object>({
	children,
	...props
}: ListBoxProps<T>) {
	return (
		<AriaListBox
			{...props}
			className={composeTailwindRenderProps(props.className, "")}
		>
			{children}
		</AriaListBox>
	);
}

export const dropdownItemStyles = tv({
	base: cn(
		"group flex items-center gap-4 cursor-default select-none",
		"py-2 pl-3 pr-3 selected:pr-1 rounded-lg outline outline-0 text-sm",
		"forced-color-adjust-none no-underline",
		"[&[href]]:cursor-pointer hover:bg-(--color-popover-item-hover-background)",
	),
	variants: {
		isDisabled: {
			false: "text-neutral-900",
			true: "text-neutral-300 forced-colors:text-[GrayText]",
		},
		isPressed: {
			true: "bg-neutral-100",
		},
		isFocused: {
			true: cn(
				"bg-(--color-popover-item-hover-background)",
				"forced-colors:bg-[Highlight] forced-colors:text-[HighlightText]",
			),
		},
	},
	compoundVariants: [
		{
			isFocused: false,
			isOpen: true,
			className: cn("bg-neutral-100"),
		},
	],
});

export function DropdownItem(props: ListBoxItemProps) {
	const textValue =
		props.textValue ||
		(typeof props.children === "string" ? props.children : undefined);
	return (
		<AriaListBoxItem
			{...props}
			textValue={textValue}
			className={dropdownItemStyles}
		>
			{composeRenderProps(props.children, (children, { isSelected }) => (
				<>
					<span
						className={cn(
							"flex items-center flex-1 gap-2 text-(--color-secondary-text) truncate group-selected:font-semibold",
						)}
					>
						{children}
					</span>
					<span
						className={cn(
							"flex items-center w-5 text-(--color-secondary-text)",
						)}
					>
						{isSelected && <Check className="w-4 h-4" />}
					</span>
				</>
			))}
		</AriaListBoxItem>
	);
}

export interface DropdownSectionProps<T> extends SectionProps<T> {
	title?: string;
	items?: any;
}

export function DropdownSection<T extends object>(
	props: DropdownSectionProps<T>,
) {
	return (
		<ListBoxSection
			className={cn(
				"first:-mt-[5px]",
				"after:content-[''] after:block after:h-[5px]",
				"last:after:hidden",
			)}
		>
			<Header
				className={cn(
					"text-sm font-semibold text-neutral-500 px-4 py-1 truncate sticky -top-[5px] -mt-px -mx-1 z-10",
					"bg-neutral-100/60 backdrop-blur-md supports-[-moz-appearance:none]:bg-neutral-100",
					"border-y border-y-neutral-200",
					"[&+*]:mt-1",
				)}
			>
				{props.title}
			</Header>
			<Collection items={props.items}>{props.children}</Collection>
		</ListBoxSection>
	);
}
