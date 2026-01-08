import { Atom, BadgeCheck, Bookmark, Egg } from "lucide-react";
import { NavLink } from "react-router";
import { cn } from "~/utils/cn";
import { RadialAvatar } from "./radiant-avatar";
import { Tooltip } from "./tooltip";

const NavigationControlItem = ({
	route,
}: {
	route: {
		label: string;
		route: string;
		shortcut: string[];
		icon: React.ReactNode;
	};
}) => {
	const isSettings = route.route === "/settings";

	return (
		<Tooltip
			key={route.label}
			contentClassName="text-xs"
			content={route.label}
			shortcuts={route.shortcut}
			trigger={
				<NavLink
					to={route.route}
					className={({ isActive }) =>
						cn(
							"text-sm flex shrink-0 items-center justify-center w-9.5 h-9.5 rounded-full hover:bg-neutral-100 transition-all duration-300",
							{
								"bg-green-900 text-green-100 hover:bg-green-800":
									isActive && !isSettings,
								"bg-neutral-200 text-neutral-800 hover:bg-neutral-200":
									isActive && isSettings,
							},
						)
					}
				>
					<li>{route.icon}</li>
				</NavLink>
			}
		/>
	);
};
const routes = [
	{
		label: "Journal",
		route: "/",
		shortcut: ["⌘", "J"],
		icon: <Egg className="size-5.5" />,
	},
	{
		label: "Tasks",
		route: "/tasks",
		shortcut: ["⌘", "T"],
		icon: <BadgeCheck className="size-5.5" />,
	},
	{
		label: "Canvas",
		route: "/canvas",
		shortcut: ["⌘", "C"],
		icon: <Atom className="size-5.5" />,
	},
	{
		label: "Bookmarks",
		route: "/bookmarks",
		shortcut: ["⌘", "B"],
		icon: <Bookmark className="size-5.5" />,
	},
	{
		label: "Settings",
		route: "/settings",
		shortcut: ["⌘", "S"],
		icon: <RadialAvatar size="sm" seed={Math.random().toString()} />,
	}
];

export const NavigationControl = () => {
	return (
		<div className="bg-white p-1.5 rounded-full absolute bottom-5 left-1/2 right-1/2 -translate-x-1/2 navigation-control w-fit z-50">
			<ul className="flex items-center justify-center gap-1 relative">
				{routes.map((route) => (
					<NavigationControlItem key={route.label} route={route} />
				))}
			</ul>
		</div>
	);
};
