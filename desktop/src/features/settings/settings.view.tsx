import {
	BrainIcon,
	Palette,
	SettingsIcon,
	SparklesIcon,
	SwatchBook,
} from "lucide-react";
import { useState } from "react";
import { Radio, RadioGroup } from "~/components/shared/radio";
import { useThemeContext } from "~/context/theme-context";
import { cn } from "~/utils/cn";
import { ModelsSection } from "./components/models.section";
import { PreferencesSection } from "./components/preferences.section";
import { WhatsNewSection } from "./components/whats-new.section";

type AvailableSections = "preferences" | "models" | "whats-new";

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
		models: {
			label: "Models",
			icon: <BrainIcon className="size-4" strokeWidth={2.5} />,
			component: <ModelsSection />,
		},
		"whats-new": {
			label: "What's New",
			icon: <SparklesIcon className="size-4" strokeWidth={2.5} />,
			component: <WhatsNewSection />,
		},
	};

	return (
		<div className="w-full h-full mx-auto grid grid-cols-24">
			<div className="col-span-8 py-8 sticky top-0 h-screen flex justify-end pr-15 bg-neutral-50">
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
			<div className="col-span-16 bg-transparent p-8">
				{sections[selectedSection as AvailableSections].component}
			</div>
		</div>
	);
};

// - preferences
//   - theme
//   - timezone
// - models
// - what's new
