import { useQueryClient } from "@tanstack/react-query";
import { getGetInboxTasksQueryKey, useGetInboxTasks } from "~/aether-sdk";
// import { appWindow } from "../journal/journal";
import { InboxTasksView } from "./inbox.view";

export const TasksView = () => {
	// appWindow.onFocusChanged(({ payload }) => {
	// 	if (payload) {
	// 		queryClient.invalidateQueries({ queryKey: inboxTasksQueryKey });
	// 	}
	// });

	return (
		<div className="grid-cols-24 grid pt-5 w-screen h-screen overflow-y-scroll pb-25">
			<div className="col-span-7" />
			<div className="col-span-10">
				<InboxTasksView />
			</div>
			<div className="col-span-7" />
		</div>
	);
};
