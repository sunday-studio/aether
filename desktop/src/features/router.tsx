import { createBrowserRouter, createRoutesFromElements, Navigate, Route } from 'react-router';
import { Layout } from '~/components/shared/layout';
// Features
import { GraphView } from './graph/graph.view';
import { Journal } from './journal/journal';
import { SettingsView } from './settings/settings.view';
import { TaskLayout } from './tasks/components/task-layout';
import { GoalView } from './tasks/goal.view';
import { InboxTasksView } from './tasks/inbox.view';
import { OverdueTasksView } from './tasks/overdue-tasks.view';

export const router = createBrowserRouter(
	createRoutesFromElements(
		<Route path='' element={<Layout />}>
			<Route index element={<Journal />} />
			<Route path='/tasks' element={<TaskLayout />}>
				<Route index element={<InboxTasksView />} />
				{/* <Route path="/tasks/all" element={<InboxTasksView />} /> */}
				<Route path='/tasks/overdue' element={<OverdueTasksView />} />
				<Route path='/tasks/goal/:goalId' element={<GoalView />} />
			</Route>
			<Route path='/canvas/*' element={<Navigate to='/' replace />} />
			<Route path='/bookmarks/*' element={<Navigate to='/' replace />} />
			<Route path='/graph' element={<GraphView />} />
			<Route path='/settings' element={<SettingsView />} />
		</Route>,
	),
);
