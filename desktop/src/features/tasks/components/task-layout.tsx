import { Outlet } from "react-router";
import { TaskSidebar } from "./task-sidebar";

export const TaskLayout = () => {
	return (
		<div className="grid grid-cols-24 w-screen h-screen overflow-hidden pb-25 relative pt-2">
			<div className="col-span-5" />
			{/* Make TaskSidebar fixed and always sticky */}
			<div className="col-span-4">
				<div className="fixed left-0 top-0 h-screen w-[16.6666667%] z-20 pt-2">
					<TaskSidebar />
				</div>
				{/* Empty spacer to maintain grid layout */}
				<div className="invisible h-screen" />
			</div>
			{/* Make main outlet area scrollable */}
			<div className="col-span-10 px-4 h-screen overflow-auto">
				<Outlet />
			</div>
			<div className="col-span-5" />
		</div>
	);
};
