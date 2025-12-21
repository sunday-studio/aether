// import { CreateGoalDialog } from "./components/goals/create-goal-dialog";
import { Outlet } from "react-router";
import { TaskSidebar } from "./task-sidebar";

export const TaskLayout = () => {
	return (
		<div className="grid grid-cols-24 pt-5 w-screen h-screen overflow-y-scroll pb-25 relative">
			<div className="col-span-4" />
			<div className="col-span-4">
				{/* Sidebar stays within the grid, but uses sticky for persistent positioning */}
				<div className="sticky top-0 h-screen">
					<TaskSidebar />
				</div>
			</div>
			<div className="col-span-10 px-4">
				<Outlet />
			</div>
			<div className="col-span-4" />
		</div>
	);
};
