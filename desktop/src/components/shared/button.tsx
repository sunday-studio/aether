import { tv } from "tailwind-variants";
import { cn } from "~/utils/cn";
import { Tooltip } from "./tooltip";

type ButtonVariant = "primary" | "secondary" | "destructive";

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
	base: "flex items-center gap-1 transition-all duration-200 cursor-pointer px-3 py-1.5 text-sm rounded-full",
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
					<p className="text-sm">{label}</p>
					{/* <div className="flex items-center justify-center gap-0.5">
						{shortcuts?.map(
							(shortcut) =>
								shortcut && (
									<kbd
										key={shortcut}
										className="px-1 bg-linear-to-b from-neutral-200 to-neutral-300 h-5 w-fit min-w-5 rounded-md text-xs font-medium pointer-events-none  inline-flex items-center justify-center gap-1 text-neutral-700 text-center select-none"
									>
										{shortcut}
									</kbd>
								),
						)}
					</div> */}
				</button>
			}
		/>
	);
};
