import { Outlet } from "react-router";
import { TaskSidebar } from "./task-sidebar";

export const TaskLayout = () => {
	return (
		<div className="grid grid-cols-24 w-screen h-screen overflow-y-scroll pb-25 relative pt-2">
			<div className="col-span-3" />
			<div className="col-span-4">
				<div className="sticky top-0 h-screen">
					<TaskSidebar />
				</div>
			</div>
			<div className="col-span-13 px-4">
				<Outlet />
			</div>
			<div className="col-span-4" />
		</div>
	);
};
