import {
	arrow,
	autoUpdate,
	FloatingArrow,
	FloatingPortal,
	flip,
	offset,
	shift,
	useClick,
	useDismiss,
	useFloating,
	useHover,
	useInteractions,
	useRole,
} from "@floating-ui/react";
import { type FC, useRef, useState } from "react";
import { cn } from "~/utils/cn";

interface TooltipProps {
	trigger: React.ReactNode;
	shortcuts?: string[];
	content: string | React.ReactElement;
	shouldFlip?: boolean;
	placement?: "top" | "right" | "bottom" | "left";
	leaveDuration?: number;
	hoverDuration?: number;
	showArrow?: boolean;
	contentClassName?: string;
	containerClassName?: string;
}

export const Tooltip: FC<TooltipProps> = ({
	trigger,
	content,
	shortcuts,
	placement = "top",
	shouldFlip = true,
	leaveDuration = 10,
	hoverDuration = 200,
	showArrow = true,
	contentClassName,
	containerClassName,
}) => {
	const [isOpen, setIsOpen] = useState(false);
	const arrowRef = useRef(null);

	const { x, y, refs, strategy, context } = useFloating({
		placement,
		open: isOpen,
		onOpenChange: setIsOpen,
		middleware: [
			offset(8),
			shouldFlip && flip(),
			shift(),
			arrow({ element: arrowRef }),
		],
		whileElementsMounted: autoUpdate,
	});

	const { getReferenceProps, getFloatingProps } = useInteractions([
		useHover(context, {
			delay: {
				open: hoverDuration,
				close: leaveDuration,
			},
		}),
		useRole(context),
		useDismiss(context),
		useClick(context),
	]);

	const contentElement =
		typeof content === "string" ? (
			<p
				className={cn(
					"text-sm font-medium text-neutral-200 flex flex-col gap-1",
					contentClassName,
				)}
			>
				{content}
				{shortcuts && shortcuts.length > 0 ? (
					<span className="text-xs text-neutral-400">
						{shortcuts.map((s) => (
							<span key={s}>{s}</span>
						))}
					</span>
				) : null}
			</p>
		) : (
			content
		);

	return (
		<>
			<span ref={refs.setReference} {...getReferenceProps()}>
				{trigger}
			</span>
			{isOpen && (
				<FloatingPortal>
					<div
						ref={refs.setFloating}
						className={cn(
							"text-sm font-medium py-1.5 px-2.5 rounded-full box-border max-w-xs shadow-1 bg-neutral-800 z-100000 inset-ring-2 inset-ring-neutral-600",
							containerClassName,
						)}
						style={{
							position: strategy,
							top: y ?? 0,
							left: x ?? 0,
						}}
						{...getFloatingProps()}
					>
						{showArrow && (
							<FloatingArrow
								ref={arrowRef}
								context={context}
								tipRadius={3}
								className="fill-neutral-800"
							/>
						)}
						{contentElement}
					</div>
				</FloatingPortal>
			)}
		</>
	);
};
