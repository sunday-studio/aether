// import { CreateGoalDialog } from "./components/goals/create-goal-dialog";
import { Outlet } from "react-router";
import { TaskSidebar } from "./task-sidebar";

export const TaskLayout = () => {
	return (
		<div className="grid-cols-24 grid pt-5 w-screen h-screen overflow-y-scroll pb-25">
			<div className="col-span-4" />
			<div className="col-span-4 sticky top-0">
				<TaskSidebar />
			</div>
			<div className="col-span-10 px-4">
				<Outlet />
			</div>
			<div className="col-span-4" />
		</div>
	);
};
