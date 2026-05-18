import { Outlet } from 'react-router';
import { TaskSidebar } from './task-sidebar';

export const TaskLayout = () => {
	return (
		<div className='relative mx-auto h-screen w-screen max-w-4xl overflow-y-scroll pt-2 pb-25'>
			<TaskSidebar />
			<div>
				<Outlet />
			</div>
		</div>
	);
};
