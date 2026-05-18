import { ChevronDown } from 'lucide-react';
import type React from 'react';
import {
	Select as AriaSelect,
	type SelectProps as AriaSelectProps,
	Button,
	ListBox,
	type ListBoxItemProps,
	Popover,
	SelectValue,
} from 'react-aria-components';
import { tv } from 'tailwind-variants';
import { cn, composeTailwindRenderProps, focusRing } from '~/utils/cn';
import { DropdownItem, DropdownSection, type DropdownSectionProps } from './dropdown';
import { Description, FieldError, Label } from './field';

const styles = tv({
	extend: focusRing,
	base: cn(
		'flex h-9 w-full min-w-[180px] items-center gap-4 rounded-xl pr-2 pl-3 transition',
		'cursor-default text-start',
		'bg-(--color-select-trigger-background)',
		'hover:bg-(--color-select-trigger-hover-background)',
		'[-webkit-tap-highlight-color:transparent]',
	),
	variants: {
		isDisabled: {
			false: cn(
				'text-neutral-800',
				'group-invalid:outline group-invalid:outline-red-600',
				'forced-colors:group-invalid:outline-[Mark]',
			),
			true: cn('hover:bg-(--color-select-trigger-background)'),
		},
	},
});

export interface SelectProps<T extends object> extends Omit<AriaSelectProps<T>, 'children'> {
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
				'group flex flex-col gap-1 relative font-sans',
			)}
		>
			{label && <Label>{label}</Label>}
			<Button className={styles({ isDisabled: props.isDisabled })}>
				<SelectValue className='flex-1 text-sm text-(--color-secondary-text)'>
					{({ selectedText, defaultChildren }) => selectedText || defaultChildren}
				</SelectValue>
				<ChevronDown
					aria-hidden
					className={cn(
						'h-4 w-4 text-(--color-secondary-text) forced-colors:text-[ButtonText]',
						'group-disabled:text-neutral-200 forced-colors:group-disabled:text-[GrayText]',
					)}
				/>
			</Button>
			{description && <Description>{description}</Description>}
			<FieldError value={errorMessage} />
			<Popover className='max-h-[200px] min-w-(--trigger-width) overflow-auto rounded-xl bg-(--color-popover-background) shadow-md ring ring-(--color-popover-ring)'>
				<ListBox
					items={items}
					className='box-border max-h-[inherit] overflow-auto p-1 outline-hidden [clip-path:inset(0_0_0_0_round_.75rem)]'
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

export function SelectSection<T extends object>(props: DropdownSectionProps<T>) {
	return <DropdownSection {...props} />;
}
