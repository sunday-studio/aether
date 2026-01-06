// TODO: add support for group tooltips

import {
	arrow,
	autoUpdate,
	FloatingArrow,
	FloatingPortal,
	flip,
	offset,
	shift,
	useFloating,
	useHover,
	useInteractions,
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
	disabled?: boolean;
}

export const Tooltip: FC<TooltipProps> = ({
	trigger,
	content,
	shortcuts,
	placement = "top",
	shouldFlip = true,
	leaveDuration = 100,
	hoverDuration = 600,
	showArrow = true,
	contentClassName,
	containerClassName,
	disabled = false,
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
		].filter(Boolean),
		whileElementsMounted: autoUpdate,
	});

	const hover = useHover(context, {
		enabled: !disabled,
		delay: {
			open: hoverDuration,
			close: leaveDuration,
		},
		move: false,
	});

	const { getReferenceProps, getFloatingProps } = useInteractions([hover]);

	const hasShortcuts = shortcuts && shortcuts.length > 0;

	const contentElement =
		typeof content === "string" ? (
			<p
				className={cn(
					"text-xs text-neutral-100 flex flex-row gap-2",
					{
						"py-0.5": hasShortcuts,
					},
					contentClassName,
				)}
			>
				{content}
				{shortcuts && shortcuts.length > 0 ? (
					<span className="text-xs text-neutral-300 flex items-center justify-start gap-1">
						{shortcuts.map((s) => (
							<span
								key={s}
								className="bg-neutral-700/70  text-white rounded-sm text-xs w-4 inline-block text-center font-mono"
							>
								{s}
							</span>
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
							"bg-neutral-950",
							"text-md  rounded-lg",
							"py-1 px-2 box-border leading-none",
							"max-w-xs shadow-1 z-1000",
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
								className="fill-neutral-950"
							/>
						)}
						{contentElement}
					</div>
				</FloatingPortal>
			)}
		</>
	);
};
