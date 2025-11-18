import { format, isToday as isTodayFn } from "date-fns";
import clsx from "clsx";
import { EntryEditor } from "./entry-editor";
import type { DbEntry } from "~/aether-sdk/models";

interface TimelineEntryProps {
	date: Date;
	todayRef: React.RefObject<HTMLDivElement>;
	data?: DbEntry[] | [];
}

export const TimelineEntry = ({ date, todayRef, data }: TimelineEntryProps) => {
	const isToday = isTodayFn(date);

	return (
		<div
			ref={isToday ? todayRef : null}
			className={clsx("mb-8 px-4", {
				"debug bg-blue-100 h-full": isToday,
				"h-32": !isToday,
			})}
		>
			<div className="sticky top-0 bg-gray-50 py-2 z-10 text-neutral-700 newreader-font">
				{format(date, "EEEE, MMMM d")}
			</div>

			<div className="space-y-2 mt-4 h-full">
				<EntryEditor data={data ?? []} />
			</div>
		</div>
	);
};
