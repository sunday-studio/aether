import { InboxTasksView } from "./inbox.view";

export const TasksView = () => {
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
