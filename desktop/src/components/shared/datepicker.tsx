import type React from "react";
import { useRef } from "react";
import {
	Button,
	CalendarCell,
	CalendarGrid,
	DatePicker,
	Dialog,
	Popover,
} from "react-aria-components";
import { Calendar } from "./calendar";

interface DateTimePickerProps {
	value: any;
	onChange: (value: any) => void;
	trigger: React.ReactNode;
}

export function DateTimePicker({
	value,
	onChange,
	trigger,
}: DateTimePickerProps) {
	const triggerRef = useRef<HTMLButtonElement>(null);

	return (
		<DatePicker
			value={value}
			onChange={onChange}
			className="inline-flex flex-col gap-2"
		>
			<div className="flex items-center gap-2 relative">
				<Button ref={triggerRef} className="relative z-10">
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
