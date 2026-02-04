import { tv } from "tailwind-variants";
import { cn } from "~/utils/cn";
import { Tooltip } from "./tooltip";

type ButtonVariant = "primary" | "destructive";

interface ButtonProps {
	onClick: () => void;
	label: string;
	tooltipContent: string;
	shortcuts?: string[];
	variant?: ButtonVariant;
	isDisabled?: boolean;
}

// bg-linear-to-b from-neutral-100 to-neutral-200
const buttonStyles = tv({
	base: "flex items-center gap-1 transition-all duration-200 cursor-pointer px-3 py-2.5 text-[13px] rounded-full leading-none",
	variants: {
		variant: {
			primary:
				"text-(--color-button-primary-foreground) bg-linear-to-b from-(--color-button-primary-start) to-(--color-button-primary-end)",
			// secondary:
			// 	"bg-(--color-button-secondary-background) text-(--color-button-secondary-foreground)",
			destructive:
				"text-(--color-button-destructive-foreground) bg-linear-to-b from-(--color-button-destructive-start) to-(--color-button-destructive-end)",
		},
		isDisabled: {
			true: "opacity-50 cursor-not-allowed",
		},
	},
});

export const Button = ({
	onClick,
	label,
	tooltipContent,
	shortcuts,
	variant = "primary",
	isDisabled = false,
}: ButtonProps) => {
	return (
		<Tooltip
			shortcuts={shortcuts}
			content={tooltipContent}
			trigger={
				<button
					className={buttonStyles({ variant, isDisabled })}
					type="button"
					onClick={onClick}
					disabled={isDisabled}
				>
					{label}
				</button>
			}
		/>
	);
};
