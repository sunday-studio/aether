import type React from "react";
import { useRef } from "react";
import { Button, DatePicker, Dialog, Popover } from "react-aria-components";
import { cn } from "tailwind-variants";
import { Calendar } from "./calendar";

interface DateTimePickerProps {
	value: any;
	onChange: (value: any) => void;
	trigger: React.ReactNode;
	className?: string;
	isDisabled?: boolean;
}

export function DateTimePicker({
	value,
	onChange,
	trigger,
	className,
	isDisabled,
}: DateTimePickerProps) {
	const triggerRef = useRef<HTMLButtonElement>(null);

	return (
		<DatePicker
			value={value}
			onChange={onChange}
			className={cn("flex flex-col gap-2", className)}
			isDisabled={isDisabled}
		>
			<div className="flex-1">
				<Button
					type="button"
					ref={triggerRef}
					className="relative z-10 h-full all leading-none p-0! m-0! w-full flex-1"
				>
					{trigger}
				</Button>

				<Popover
					className="absolute left-0 top-full mt-2 z-50 min-w-[240px] rounded-xl bg-white p-3 shadow-xl bg-linear-to-bl from-neutral-800 text-neutral-600 to-neutral-950 "
					style={{
						transform: "translateY(0)",
					}}
					shouldFlip
					triggerRef={triggerRef}
				>
					<Dialog>
						<Calendar />
					</Dialog>
				</Popover>
			</div>
		</DatePicker>
	);
}
