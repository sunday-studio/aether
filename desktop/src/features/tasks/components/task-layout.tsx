import { Outlet } from "react-router";
import { TaskSidebar } from "./task-sidebar";

export const TaskLayout = () => {
	return (
		<div className="grid grid-cols-24 w-full min-h-full pb-25 relative pt-2">
			<div className="col-span-5" />
			<div className="col-span-4">
				<div className="sticky top-2 self-start">
					<TaskSidebar />
				</div>
			</div>
			<div className="col-span-10 px-4">
				<Outlet />
			</div>
			<div className="col-span-5" />
		</div>
	);
};
