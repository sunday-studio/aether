import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getGetEntriesQueryKey } from "~/aether-sdk";
import { JournalTimeline } from "./components/journal-timeline";
import { JournalGridView } from "./components/journal-grid-view";
import { LayoutGrid, List } from "lucide-react";
import { cn } from "~/utils/cn";

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
			{/* View Toggle */}
			<div className="absolute top-4 right-4 z-20 flex items-center gap-1 bg-white rounded-lg border border-neutral-200 p-1 shadow-sm">
				<button
					type="button"
					onClick={() => setViewMode("timeline")}
					className={cn(
						"p-2 rounded-md transition-colors",
						viewMode === "timeline"
							? "bg-neutral-900 text-white"
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
