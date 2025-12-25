import { forwardRef } from "react";
import { cn } from "~/utils/cn";

interface TaskActionButtonProps {
	children: React.ReactNode;
	className?: string;
}

export const TaskActionButton = forwardRef<
	HTMLSpanElement,
	TaskActionButtonProps
>(({ children, className }, ref) => {
	return (
		<span
			ref={ref}
			className={cn(
				"w-6 h-6 rounded-lg",
				"bg-neutral-200 text-neutral-400 text-sm",
				"flex items-center justify-center",
				"focus:ring-2 focus:ring-offset-1 focus:ring-neutral-300",
				"active:bg-neutral-300 active:ring-2 active:ring-offset-1 active:ring-neutral-300",
				"transition-colors",
				"hover:bg-neutral-300 hover:text-neutral-500 cursor-pointer",
				className,
			)}
		>
			{children}
		</span>
	);
});
