import { useQueryClient } from "@tanstack/react-query";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LayoutGrid, List } from "lucide-react";
import { useState } from "react";
import { getGetEntriesQueryKey } from "~/aether-sdk";
import { cn } from "~/utils/cn";
import { JournalGridView } from "./components/journal-grid-view";
import { JournalTimeline } from "./components/journal-timeline";

// export const appWindow = getCurrentWindow();

type ViewMode = "timeline" | "grid";

export const Journal = () => {
	const queryClient = useQueryClient();
	const entriesQueryKey = getGetEntriesQueryKey();
	const [viewMode, setViewMode] = useState<ViewMode>("timeline");

	// appWindow.onFocusChanged(({ payload }) => {
	// 	if (payload) {
	// 		queryClient.invalidateQueries({ queryKey: entriesQueryKey });
	// 	}
	// });

	return (
		<main className="w-screen h-screen relative">
			<div className="absolute top-4 right-4 z-20 flex items-center gap-1  rounded-lg border border-neutral-200 p-1 shadow-sm">
				<button
					type="button"
					onClick={() => setViewMode("timeline")}
					className={cn(
						"p-2 rounded-md transition-colors",
						viewMode === "timeline"
							? "bg-(--color-primary) text-white"
							: "text-neutral-600 hover:bg-neutral-100",
					)}
					title="Timeline view"
				>
					<List className="w-4 h-4" />
				</button>
				<button
					type="button"
					onClick={() => setViewMode("grid")}
					className={cn(
						"p-2 rounded-md transition-colors",
						viewMode === "grid"
							? "bg-neutral-900 text-white"
							: "text-neutral-600 hover:bg-neutral-100",
					)}
					title="Grid view"
				>
					<LayoutGrid className="w-4 h-4" />
				</button>
			</div>

			{viewMode === "timeline" ? <JournalTimeline /> : <JournalGridView />}
		</main>
	);
};
