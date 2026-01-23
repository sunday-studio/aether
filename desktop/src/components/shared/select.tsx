import { ChevronDown } from "lucide-react";
import type React from "react";
import {
	Select as AriaSelect,
	type SelectProps as AriaSelectProps,
	Button,
	ListBox,
	type ListBoxItemProps,
	Popover,
	SelectValue,
} from "react-aria-components";
import { tv } from "tailwind-variants";
import { cn, composeTailwindRenderProps, focusRing } from "~/utils/cn";
import {
	DropdownItem,
	DropdownSection,
	type DropdownSectionProps,
} from "./dropdown";
import { Description, FieldError, Label } from "./field";

const styles = tv({
	extend: focusRing,
	base: cn(
		"flex items-center gap-4 w-full min-w-[180px] h-9 pl-3 pr-2 rounded-xl transition",
		"text-start cursor-default",
		"bg-(--color-select-trigger-background)",
		"hover:bg-(--color-select-trigger-hover-background)",
		"[-webkit-tap-highlight-color:transparent]",
	),
	variants: {
		isDisabled: {
			false: cn(
				"text-neutral-800",
				"group-invalid:outline group-invalid:outline-red-600",
				"forced-colors:group-invalid:outline-[Mark]",
			),
			true: cn("hover:bg-(--color-select-trigger-background)"),
		},
	},
});

export interface SelectProps<T extends object>
	extends Omit<AriaSelectProps<T>, "children"> {
	label?: string;
	description?: string;
	errorMessage?: string;
	items?: Iterable<T>;
	children: React.ReactNode | ((item: T) => React.ReactNode);
}

export function Select<T extends object>({
	label,
	description,
	errorMessage,
	children,
	items,
	...props
}: SelectProps<T>) {
	return (
		<AriaSelect
			{...props}
			className={composeTailwindRenderProps(
				props.className,
				"group flex flex-col gap-1 relative font-sans",
			)}
		>
			{label && <Label>{label}</Label>}
			<Button className={styles({ isDisabled: props.isDisabled })}>
				<SelectValue className="flex-1 text-sm font-medium text-(--color-secondary-text)">
					{({ selectedText, defaultChildren }) =>
						selectedText || defaultChildren
					}
				</SelectValue>
				<ChevronDown
					aria-hidden
					className={cn(
						"w-4 h-4 text-(--color-secondary-text) forced-colors:text-[ButtonText]",
						"group-disabled:text-neutral-200 forced-colors:group-disabled:text-[GrayText]",
					)}
				/>
			</Button>
			{description && <Description>{description}</Description>}
			<FieldError value={errorMessage} />
			<Popover className="min-w-(--trigger-width) max-h-[200px] overflow-auto bg-(--color-popover-background) rounded-xl ring ring-(--color-popover-ring) shadow-md">
				<ListBox
					items={items}
					className="outline-hidden box-border p-1 max-h-[inherit] overflow-auto [clip-path:inset(0_0_0_0_round_.75rem)]"
				>
					{children}
				</ListBox>
			</Popover>
		</AriaSelect>
	);
}

export function SelectItem(props: ListBoxItemProps) {
	return <DropdownItem {...props} />;
}

export function SelectSection<T extends object>(
	props: DropdownSectionProps<T>,
) {
	return <DropdownSection {...props} />;
}
