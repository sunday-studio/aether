import { clsx } from "clsx";
import type React from "react";
import { forwardRef, type ReactNode, type Ref } from "react";
import { cn } from "~/utils/cn";

export function Root({
	children,
	className,
}: {
	children: ReactNode;
	className?: string;
}) {
	return (
		<ol className={clsx("relative ", className)} aria-label="Timeline">
			{children}
		</ol>
	);
}

function TimelineItem({
	indicator = <Timeline.Indicator />,
	leftContent,
	rightContent,
	leftContainerClassName,
	rightContainerClassName,
	className,
	indicatorContainerClassName,
}: {
	indicator?: React.ReactNode;
	leftContent?: React.ReactNode;
	rightContent: React.ReactNode;
	rightContainerClassName?: string;
	leftContainerClassName?: string;
	indicatorContainerClassName?: string;
	className?: string;
}) {
	return (
		<div className={cn("flex group", className)}>
			<div className={cn("left", leftContainerClassName)}>{leftContent}</div>
			{indicator}
			<div
				className={cn("indicator h-full", indicatorContainerClassName)}
			></div>
			<div className={cn("right w-full", rightContainerClassName)}>
				{rightContent}
			</div>
		</div>
	);
}

const Indicator = forwardRef<
	HTMLDivElement,
	{
		children?: ReactNode;
		className?: string;
		containerClassName?: string;
		onClick?: () => void;
	}
>(({ children, className, containerClassName, onClick = () => {} }, ref) => {
	return (
		<div
			className={cn(
				"relative flex flex-col items-center self-stretch justify-start",
				containerClassName,
			)}
		>
			<div
				aria-hidden="true"
				className="flex h-full w-7 flex-col items-center justify-start"
			>
				<button
					type="button"
					ref={ref as Ref<HTMLButtonElement>}
					onClick={onClick}
					style={{
						clipPath:
							"polygon(50% 0%, 90% 20%, 100% 60%, 75% 100%, 25% 100%, 0% 60%, 10% 20%)",
					}}
					className={clsx(
						"flex h-4.5 w-4.5 items-center justify-center bg-neutral-200",
						className,
					)}
				>
					{children}
				</button>
				<div className="my-1 w-[2px] flex-1 shrink-0 bg-neutral-200 group-last:hidden" />
			</div>
		</div>
	);
});

Indicator.displayName = "Timeline.Indicator";

const LeftContent = ({
	children,
	className,
}: {
	children: ReactNode;
	className?: string;
}) => {
	return (
		<div className={cn("flex flex-1 justify-end", className)}>{children}</div>
	);
};

LeftContent.displayName = "Timeline.LeftContent";

const RightContent = ({
	children,
	className,
}: {
	children: ReactNode;
	className?: string;
}) => {
	return (
		<div className={cn("flex flex-1 justify-start", className)}>{children}</div>
	);
};

RightContent.displayName = "Timeline.RightContent";

export const Timeline = Object.assign(Root, {
	Item: TimelineItem,
	LeftContent,
	RightContent,
	Indicator,
});
