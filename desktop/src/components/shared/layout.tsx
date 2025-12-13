import { Outlet } from "react-router";
import { useRegisterShortcuts } from "~/hooks/use-register-shortcuts";
import { NavigationControl } from "./navigation-control";

export const Layout = () => {
	useRegisterShortcuts();

	return (
		<div className="w-screen h-screen  relative">
			<Outlet />
			<NavigationControl />
		</div>
	);
};
