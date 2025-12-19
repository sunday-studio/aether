import { Atom, BadgeCheck, Egg } from "lucide-react";
import { useMemo } from "react";
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
		icon: React.ReactNode;
	};
}) => {
	const isSettings = route.route === "/settings";

	return (
		<Tooltip
			key={route.label}
			content={route.label}
			trigger={
				<NavLink
					to={route.route}
					className={({ isActive }) =>
						cn(
							"flex shrink-0 items-center justify-center w-10 h-10 rounded-full hover:bg-neutral-100 transition-all duration-300",
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

export const NavigationControl = () => {
	const seed = useMemo(() => Math.random().toString(), []);
	const routes = [
		{
			label: "Journal",
			route: "/",
			// shortcut: [`${Key.Meta}+j`],
			icon: <Egg className="size-6" />,
		},
		{
			label: "Tasks",
			route: "/tasks",
			// shortcut: [`${Key.Meta}+j`],
			icon: <BadgeCheck className="size-6" />,
		},
		{
			label: "Canvas",
			route: "/canvas",
			icon: <Atom className="size-6" />,
		},

		{
			label: "Settings",
			route: "/settings",
			icon: <RadialAvatar size="sm" seed={seed} />,
		},
	];

	return (
		<div className="bg-white p-1.5 rounded-full absolute bottom-5 left-1/2 right-1/2 -translate-x-1/2 navigation-control w-fit z-50">
			<ul className="flex items-center justify-center gap-1">
				{routes.map((route) => (
					<NavigationControlItem key={route.label} route={route} />
				))}
			</ul>
		</div>
	);
};
