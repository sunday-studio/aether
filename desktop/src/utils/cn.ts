import { type ClassValue, clsx } from "clsx";
import { composeRenderProps } from "react-aria-components";
import { twMerge } from "tailwind-merge";
import { tv } from "tailwind-variants";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

export function composeTailwindRenderProps<T>(
	className: string | ((v: T) => string) | undefined,
	tw: string,
): string | ((v: T) => string) {
	return composeRenderProps(className, (className) => twMerge(tw, className));
}

export function focusRing() {
	return tv({
		base: "outline-hidden focus-visible:outline-hidden focus-visible:ring-4",
		variants: {
			isFocusVisible: {
				false: "focus-visible:ring-transparent",
				true: "focus-visible:ring-brand-bold-default/20",
			},
		},
	});
}
