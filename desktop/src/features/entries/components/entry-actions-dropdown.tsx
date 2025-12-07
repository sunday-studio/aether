import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import { Archive, Pin, Tag, Trash } from "lucide-react";
import type { DbEntry } from "~/aether-sdk/models";
import { cn } from "~/utils/cn";

interface EntryActionsDropdownProps {
	entry: DbEntry;
	isOpen: boolean;
	onOpenChange: (open: boolean) => void;
	children: React.ReactNode;
}

const dropdownContentStyles = cn(
	"z-50 shadow-lg",
	"max-h-(--radix-dropdown-menu-content-available-height) min-w-[12rem]",
	"origin-(--radix-dropdown-menu-content-transform-origin)",
	"overflow-x-hidden overflow-y-auto",
	"rounded-lg",
	"bg-neutral-900 p-1 text-neutral-950",
	"data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2",
	"data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95",
	"data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95",
);

const dropdownItemStyles = cn(
	"relative flex items-center gap-2 text-neutral-200",
	"rounded-md px-2 py-1.5 text-sm",
	"cursor-default outline-hidden select-none",
	"focus:bg-neutral-800",
	"data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
	"data-[inset]:pl-8",
	"data-[variant=destructive]:text-red-500 data-[variant=destructive]:focus:bg-red-500/10 data-[variant=destructive]:focus:text-red-500 data-[variant=destructive]:*:[svg]:!text-destructive",
	"[&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 [&_svg:not([class*='text-'])]:text-neutral-400",
);

function DropdownMenuItem({
	className,
	inset,
	variant = "default",
	...props
}: React.ComponentProps<typeof DropdownMenu.Item> & {
	inset?: boolean;
	variant?: "default" | "destructive";
}) {
	return (
		<DropdownMenu.Item
			data-slot="dropdown-menu-item"
			data-inset={inset}
			data-variant={variant}
			className={cn(dropdownItemStyles, className)}
			{...props}
		/>
	);
}

export const EntryActionsDropdown = ({
	isOpen,
	onOpenChange,
	children,
}: EntryActionsDropdownProps) => {
	return (
		<DropdownMenu.Root open={isOpen} onOpenChange={onOpenChange}>
			<DropdownMenu.Trigger asChild>{children}</DropdownMenu.Trigger>
			<DropdownMenu.Portal>
				<DropdownMenu.Content className={dropdownContentStyles} sideOffset={5}>
					<DropdownMenu.Item className={dropdownItemStyles}>
						<Tag className="mr-2 size-4" />
						Add tags
					</DropdownMenu.Item>

					<DropdownMenu.Item className={dropdownItemStyles}>
						<Pin className="mr-2 size-4" />
						Pin
					</DropdownMenu.Item>

					<DropdownMenu.Item className={dropdownItemStyles}>
						<Archive className="mr-2 size-4" />
						Archive
					</DropdownMenu.Item>

					<DropdownMenuItem variant="destructive">
						<Trash className="mr-2 size-4" />
						Delete entry
					</DropdownMenuItem>
				</DropdownMenu.Content>
			</DropdownMenu.Portal>
		</DropdownMenu.Root>
	);
};
