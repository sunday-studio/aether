import { cn } from "~/utils/cn";

interface TaskActionButtonProps {
	children: React.ReactNode;
}

export const TaskActionButton = ({ children }: TaskActionButtonProps) => {
	return (
		<span
			className={cn(
				"w-6 h-6 rounded-lg",
				"bg-neutral-200 text-neutral-400 text-sm",
				"flex items-center justify-center",
				"focus:outline-2 focus:outline-offset-1 focus:outline-neutral-300",
				"active:bg-neutral-300 active:outline-2 active:outline-offset-1 active:outline-neutral-300",
				"transition-colors",
			)}
		>
			{children}
		</span>
	);
};
