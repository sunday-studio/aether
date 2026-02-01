import { CloudIcon, Palette, SparklesIcon } from "lucide-react";
import { useState } from "react";
// import { cn } from "~/utils/cn";
// import { ModelsSection } from "./components/models.section";
import { PreferencesSection } from "./components/preferences.section";
import { SyncSection } from "./components/sync.section";
import { WhatsNewSection } from "./components/whats-new.section";
import { cn } from "~/utils/cn";

type AvailableSections = "preferences" | "whats-new" | "sync";

export const SettingsView = () => {
	const [selectedSection, setSelectedSection] =
		useState<AvailableSections>("preferences");

	const sections: Record<
		AvailableSections,
		{ label: string; icon: React.ReactNode; component: React.ReactNode }
	> = {
		preferences: {
			label: "Preferences",
			icon: <Palette className="size-4" strokeWidth={2.5} />,
			component: <PreferencesSection />,
		},
		sync: {
			label: "Sync",
			icon: <CloudIcon className="size-4" strokeWidth={2.5} />,
			component: <SyncSection />,
		},
		// models: {
		// 	label: "Models",
		// 	icon: <BrainIcon className="size-4" strokeWidth={2.5} />,
		// 	component: <ModelsSection />,
		// },
		"whats-new": {
			label: "What's new",
			icon: <SparklesIcon className="size-4" strokeWidth={2.5} />,
			component: <WhatsNewSection />,
		},
	};

	return (
		<div className="w-full h-full mx-auto grid grid-cols-24">
			<div className="col-span-8 py-10 sticky top-0 h-screen flex justify-end pr-15 bg-(--color-background-secondary)">
				<div className="flex flex-col gap-1 items-start">
					{Object.entries(sections).map(([section, { label, icon }]) => {
						const isSelected = selectedSection === section;
						return (
							<button
								key={section}
								onClick={() => setSelectedSection(section as AvailableSections)}
								type="button"
								className={cn(
									"flex items-center gap-2 cursor-pointer text-sm text-(--color-secondary-text) hover:text-(--color-secondary-text-hover) py-1.5 px-2.5",
									{
										"text-(--color-active-text) hover:text-(--color-active-text-hover)":
											isSelected,
										"hover:text-(--color-secondary-text-hover)": !isSelected,
									},
								)}
							>
								{icon}
								<p className="text-xs">{label}</p>
							</button>
						);
					})}
				</div>
			</div>
			<div className="col-span-16 bg-transparent py-10 px-15">
				{sections[selectedSection as AvailableSections].component}
			</div>
		</div>
	);
};
