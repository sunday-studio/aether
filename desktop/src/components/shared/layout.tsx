import { Outlet, useLocation } from "react-router";
import { useRegisterShortcuts } from "~/hooks/use-register-shortcuts";
import { NavigationControl } from "./navigation-control";

export const Layout = () => {
	const location = useLocation();
	useRegisterShortcuts();
	return (
		<div className="w-screen h-screen relative overflow-hidden">
			<div
				className="h-12  flex items-center justify-center pl-14 pr-3"
				data-tauri-drag-region
			>
				<div className="text-center">{location.pathname}</div>
			</div>
			<div
				className="w-full h-full overflow-y-auto"
				style={{
					maskImage: "linear-gradient(to bottom, transparent, black 25px)",
					WebkitMaskImage:
						"linear-gradient(to bottom, transparent, black 25px)",
				}}
			>
				<NavigationControl />
				<Outlet />
			</div>
		</div>
	);
};
