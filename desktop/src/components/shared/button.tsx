import { tv } from 'tailwind-variants';
import { cn } from '~/utils/cn';
import { Tooltip } from './tooltip';

type ButtonVariant = 'primary' | 'destructive' | 'ghost' | 'secondary';

interface ButtonProps {
	onClick: () => void;
	label: string;
	tooltipContent: string;
	shortcuts?: string[];
	variant?: ButtonVariant;
	isDisabled?: boolean;
	iconLeft?: React.ReactNode;
	className?: string;
}

// bg-linear-to-b from-neutral-100 to-neutral-200
const buttonStyles = tv({
	base: 'flex cursor-pointer items-center justify-center gap-1.5 rounded-full px-3 py-2.5 text-[13px] leading-none transition-all duration-200',
	variants: {
		variant: {
			primary:
				'bg-linear-to-b from-(--color-button-primary-start) to-(--color-button-primary-end) text-(--color-button-primary-foreground)',
			secondary:
				'border border-(--color-border) bg-(--color-panel) text-(--color-primary-text) hover:bg-(--color-background-secondary)',
			ghost:
				'text-(--color-secondary-text) hover:bg-(--color-background-secondary) hover:text-(--color-active-text)',
			destructive:
				'bg-linear-to-b from-(--color-button-destructive-start) to-(--color-button-destructive-end) text-(--color-button-destructive-foreground)',
		},
		isDisabled: {
			true: 'cursor-not-allowed opacity-50',
		},
	},
});

export const Button = ({
	onClick,
	label,
	tooltipContent,
	shortcuts,
	variant = 'primary',
	isDisabled = false,
	iconLeft,
	className,
}: ButtonProps) => {
	return (
		<Tooltip
			shortcuts={shortcuts}
			content={tooltipContent}
			trigger={
				<button
					className={cn(buttonStyles({ variant, isDisabled }), className)}
					type='button'
					onClick={onClick}
					disabled={isDisabled}
				>
					{iconLeft}
					{label}
				</button>
			}
		/>
	);
};
