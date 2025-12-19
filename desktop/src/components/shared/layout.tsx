import { Outlet } from "react-router";
import { useRegisterShortcuts } from "~/hooks/use-register-shortcuts";
import { NavigationControl } from "./navigation-control";

export const Layout = () => {
	useRegisterShortcuts();

	return (
		<div className="w-screen h-screen relative overflow-hidden">
			<div className="w-full h-full">
				<NavigationControl />
				<Outlet />
			</div>
		</div>
	);
};
