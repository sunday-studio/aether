import { format, isToday as isTodayFn } from "date-fns";
import clsx from "clsx";
import { EntryEditor } from "./entry-editor";

interface TimelineEntryProps {
	date: Date;
	todayRef: React.RefObject<HTMLDivElement>;
}

export const TimelineEntry = ({ date, todayRef }: TimelineEntryProps) => {
	const isToday = isTodayFn(date);

	return (
		<div
			ref={isToday ? todayRef : null}
			className={clsx("mb-8", {
				"debug bg-blue-100 h-full": isToday,
			})}
		>
			<div className="sticky top-0 bg-gray-50 py-2 z-10 font-semibold text-neutral-600 newreader-font">
				{format(date, "EEEE, MMMM d")}
			</div>

			<div className="space-y-2 mt-4 h-full">
				<EntryEditor />
			</div>
		</div>
	);
};
