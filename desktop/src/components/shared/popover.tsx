import * as RadixPopover from "@radix-ui/react-popover";
import type * as React from "react";

interface PopoverProps {
	trigger: React.ReactNode;
	content: React.ReactNode;
	open?: boolean;
	onOpenChange?: (open: boolean) => void;
	side?: "top" | "right" | "bottom" | "left";
	sideOffset?: number;
	contentClassName?: string;
}

export const Popover: React.FC<PopoverProps> = ({
	trigger,
	content,
	open,
	onOpenChange,
	side = "bottom",
	sideOffset = 4,
	contentClassName,
}) => {
	return (
		<RadixPopover.Root open={open} onOpenChange={onOpenChange}>
			<RadixPopover.Trigger asChild>{trigger}</RadixPopover.Trigger>
			<RadixPopover.Portal>
				<RadixPopover.Content
					className={contentClassName}
					side={side}
					sideOffset={sideOffset}
				>
					{content}
				</RadixPopover.Content>
			</RadixPopover.Portal>
		</RadixPopover.Root>
	);
};
