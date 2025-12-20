import { parseDate } from "@internationalized/date";
import { ChevronLeft, ChevronRight } from "lucide-react";
import React from "react";
import {
	Button as AriaButton,
	Calendar as AriaCalendar,
	CalendarGridHeader as AriaCalendarGridHeader,
	type CalendarProps as AriaCalendarProps,
	type ButtonProps,
	CalendarCell,
	CalendarGrid,
	CalendarGridBody,
	CalendarHeaderCell,
	composeRenderProps,
	type DateValue,
	Heading,
	Text,
	useLocale,
} from "react-aria-components";
import { twMerge } from "tailwind-merge";
import { tv } from "tailwind-variants";

export const focusRing = tv({
	base: "outline-hidden focus-visible:outline-hidden focus-visible:ring-4",
	variants: {
		isFocusVisible: {
			false: "focus-visible:ring-transparent",
			true: "focus-visible:ring-brand-bold-default/20",
		},
	},
});

export function composeTailwindRenderProps<T>(
	className: string | ((v: T) => string) | undefined,
	tw: string,
): string | ((v: T) => string) {
	return composeRenderProps(className, (className) => twMerge(tw, className));
}

// Forward the aria-label from props properly, and add aria-labels to unlabeled controls
const Button = ({ children, ...props }: ButtonProps) => {
	return (
		<AriaButton
			{...props}
			className="hover:bg-neutral-700 rounded-full w-8 h-8 flex items-center justify-center hover:shadow-sm hover:text-neutral-200"
		>
			{children}
		</AriaButton>
	);
};

const cellStyles = tv({
	extend: focusRing,
	base: " w-[calc(100cqw/7)] aspect-square text-sm cursor-default rounded-full flex items-center justify-center forced-color-adjust-none [-webkit-tap-highlight-color:transparent]",
	variants: {
		isSelected: {
			false:
				"text-neutral-900  text-neutral-200 hover:bg-neutral-800 hover:shadow-xl pressed:bg-neutral-300 dark:pressed:bg-neutral-600",
			true: "bg-brand-bold-default invalid:bg-red-600 text-white forced-colors:bg-[Highlight] forced-colors:invalid:bg-[Mark] forced-colors:text-[HighlightText]",
		},
		isDisabled: {
			true: "text-neutral-300 dark:text-neutral-600 forced-colors:text-[GrayText]",
		},
	},
});

export interface CalendarProps<T extends DateValue>
	extends Omit<AriaCalendarProps<T>, "visibleDuration"> {
	errorMessage?: string;
}

export function Calendar<T extends DateValue>({
	errorMessage,
	...props
}: CalendarProps<T>) {
	return (
		<AriaCalendar
			// defaultValue={parseDate("2025-12-20" as unknown as string)}
			{...props}
			className={composeTailwindRenderProps(
				props.className,
				"flex flex-col w-[calc(10*var(--spacing)*7)] max-w-full @container ",
			)}
			// Provide accessibility label if none is provided from above
			aria-label={props["aria-label"] || props["aria-labelledby"] || "Calendar"}
		>
			<CalendarHeader />
			<CalendarGrid className="border-spacing-0">
				<CalendarGridHeader />
				<CalendarGridBody>
					{(date) => <CalendarCell date={date} className={cellStyles} />}
				</CalendarGridBody>
			</CalendarGrid>
			{errorMessage && (
				<Text slot="errorMessage" className="text-sm text-red-600">
					{errorMessage}
				</Text>
			)}
		</AriaCalendar>
	);
}

export function CalendarHeader() {
	const { direction } = useLocale();

	return (
		<header className="flex items-center gap-1 pb-4 px-1 border-box">
			<Button slot="previous" aria-label="Previous month">
				{direction === "rtl" ? (
					<ChevronRight aria-hidden size={18} />
				) : (
					<ChevronLeft aria-hidden size={18} />
				)}
			</Button>
			<Heading className="flex-1 font-sans font-semibold [font-variation-settings:normal] text-base text-center mx-2 my-0 text-neutral-200 " />
			<Button slot="next" aria-label="Next month">
				{direction === "rtl" ? (
					<ChevronLeft aria-hidden size={18} />
				) : (
					<ChevronRight aria-hidden size={18} />
				)}
			</Button>
		</header>
	);
}

export function CalendarGridHeader() {
	return (
		<AriaCalendarGridHeader>
			{(day) => (
				<CalendarHeaderCell className="text-xs text-neutral-500 font-medium">
					{day}
				</CalendarHeaderCell>
			)}
		</AriaCalendarGridHeader>
	);
}
