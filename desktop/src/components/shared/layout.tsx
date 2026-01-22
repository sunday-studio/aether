import { Outlet, useLocation } from "react-router";
import { useRegisterShortcuts } from "~/hooks/use-register-shortcuts";
import { ActivityHeatmap } from "./activity-heatmap";
import { CommandPalette } from "./command-palette";
import { NavigationControl } from "./navigation-control";

export const Layout = () => {
	const location = useLocation();
	const isDev = import.meta.env.DEV;
	const { commandPaletteOpen, setCommandPaletteOpen } = useRegisterShortcuts();
	return (
		<div className="w-screen h-screen relative overflow-hidden">
			<div
				className="h-12  flex items-center justify-center pl-14 pr-3"
				data-tauri-drag-region
			>
				{isDev && <div className="text-sm">{location.pathname}</div>}
			</div>
			<div
				className="w-full h-full overflow-y-auto"
				style={{
					maskImage: "linear-gradient(to bottom, transparent, black 25px)",
					WebkitMaskImage:
						"linear-gradient(to bottom, transparent, black 25px)",
				}}
			>
				<ActivityHeatmap />
				<NavigationControl />
				<Outlet />
			</div>
			<CommandPalette
				open={commandPaletteOpen}
				onOpenChange={setCommandPaletteOpen}
			/>
		</div>
	);
};
