import {
	createBrowserRouter,
	createRoutesFromElements,
	Route,
} from "react-router";
import { Layout } from "~/components/shared/layout";
import { CanvasView } from "./canvas/canvas.view";
// Features
import { Journal } from "./journal/journal";
import { SettingsView } from "./settings/settings.view";
import { TaskLayout } from "./tasks/components/task-layout";
import { GoalView } from "./tasks/goal.view";
import { TasksView } from "./tasks/tasks.view";

export const router = createBrowserRouter(
	createRoutesFromElements(
		<Route path="" element={<Layout />}>
			<Route index element={<Journal />} />
			<Route path="/tasks" element={<TaskLayout />}>
				<Route index element={<TasksView />} />
				<Route path="/tasks/goal/:goalId" element={<GoalView />} />
			</Route>
			<Route path="/canvas" element={<CanvasView />} />
			<Route path="/settings" element={<SettingsView />} />
		</Route>,
	),
);
