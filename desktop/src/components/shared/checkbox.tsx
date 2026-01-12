import { Check, Minus } from "lucide-react";
import {
	Checkbox as AriaCheckbox,
	type CheckboxProps,
	composeRenderProps,
} from "react-aria-components";
import { tv } from "tailwind-variants";
import { focusRing } from "~/utils/cn";

const checkboxStyles = tv({
	base: "flex gap-2 items-center group font-sans text-sm transition relative [-webkit-tap-highlight-color:transparent] text-neutral-800",
	variants: {
		isDisabled: {
			false: "text-neutral-800",
			true: "text-neutral-300 forced-colors:text-[GrayText]",
		},
	},
});

const boxStyles = tv({
	extend: focusRing,
	base: "w-4 h-4 box-border shrink-0  rounded-md flex items-center justify-center ring-2 transition",
	variants: {
		isSelected: {
			false:
				"bg-white ring-(--color) [--color:var(--color-neutral-400)] group-pressed:[--color:var(--color-neutral-500)]",
			true: "bg-(--color) ring-(--color) [--color:var(--color-neutral-700)] group-pressed:[--color:var(--color-neutral-800)] forced-colors:[--color:Highlight]!",
		},
		isInvalid: {
			true: "[--color:var(--color-red-700)] forced-colors:[--color:Mark]! group-pressed:[--color:var(--color-red-800)]",
		},
		isDisabled: {
			true: "[--color:var(--color-neutral-200)] forced-colors:[--color:GrayText]!",
		},
	},
});

const iconStyles =
	"w-3.5 h-3.5 text-white group-disabled:text-neutral-400 forced-colors:text-[HighlightText]";

export function Checkbox(props: CheckboxProps) {
	return (
		<AriaCheckbox
			{...props}
			className={composeRenderProps(props.className, (className, renderProps) =>
				checkboxStyles({ ...renderProps, className }),
			)}
		>
			{composeRenderProps(
				props.children,
				(children, { isSelected, isIndeterminate, ...renderProps }) => (
					<>
						<div
							className={boxStyles({
								isSelected: isSelected || isIndeterminate,
								...renderProps,
							})}
						>
							{isIndeterminate ? (
								<Minus aria-hidden className={iconStyles} />
							) : isSelected ? (
								<Check aria-hidden className={iconStyles} />
							) : null}
						</div>
						{children}
					</>
				),
			)}
		</AriaCheckbox>
	);
}
