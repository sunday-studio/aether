import { useQueryClient } from "@tanstack/react-query";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getGetEntriesQueryKey } from "~/aether-sdk";
import { JournalTimeline } from "./components/journal-timeline";

const appWindow = getCurrentWindow();

export const Journal = () => {
	const queryClient = useQueryClient();
	const entriesQueryKey = getGetEntriesQueryKey();

	appWindow.onFocusChanged(({ payload }) => {
		if (payload) {
			queryClient.invalidateQueries({ queryKey: entriesQueryKey });
		}
	});

	return (
		<main className="w-screen h-screen">
			<JournalTimeline />
		</main>
	);
};
