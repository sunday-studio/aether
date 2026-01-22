import { Radio, RadioGroup } from "~/components/shared/radio";
import { useThemeContext } from "~/context/theme-context";

export const SettingsView = () => {
	const {
		interfaceTheme,
		themeLight,
		themeDark,
		setInterfaceTheme,
		setThemeLight,
		setThemeDark,
		isLoading,
	} = useThemeContext();

	return (
		<div className="max-w-4xl mx-auto p-8">
			<div className="mb-8">
				<h1 className="text-3xl font-semibold text-neutral-900 mb-2">
					Settings
				</h1>
				<p className="text-neutral-600">
					Customize your app appearance and preferences
				</p>
			</div>

			<div className="space-y-8">
				{/* Theme Mode Section */}
				<section className="bg-white rounded-xl border border-neutral-200 p-6 shadow-sm">
					<div className="mb-6">
						<h2 className="text-xl font-semibold text-neutral-900 mb-1">
							Appearance
						</h2>
						<p className="text-sm text-neutral-600">
							Choose how the app looks and feels
						</p>
					</div>

					<div className="space-y-6">
						{/* Interface Theme */}
						<div>
							<RadioGroup
								label="Theme Mode"
								description="Choose between light, dark, or system preference"
								value={interfaceTheme}
								onChange={(value) =>
									setInterfaceTheme(value as "light" | "dark" | "system")
								}
								isDisabled={isLoading}
							>
								<Radio value="light">Light</Radio>
								<Radio value="dark">Dark</Radio>
								<Radio value="system">System</Radio>
							</RadioGroup>
						</div>

						{/* Light Theme Variant */}
						{interfaceTheme === "light" || interfaceTheme === "system" ? (
							<div>
								<RadioGroup
									label="Light Theme"
									description="Choose a light theme variant"
									value={themeLight}
									onChange={(value) =>
										setThemeLight(value as "light" | "amber")
									}
									isDisabled={isLoading}
								>
									<Radio value="light">Light (Neutral & White)</Radio>
									<Radio value="amber">Amber (Warm & White)</Radio>
								</RadioGroup>
							</div>
						) : null}

						{/* Dark Theme Variant */}
						{interfaceTheme === "dark" || interfaceTheme === "system" ? (
							<div>
								<RadioGroup
									label="Dark Theme"
									description="Choose a dark theme variant"
									value={themeDark}
									onChange={(value) => setThemeDark(value as "dark" | "lime")}
									isDisabled={isLoading}
								>
									<Radio value="dark">Dark (Neutral & Black)</Radio>
									<Radio value="lime">Lime (Lime & Dark Gray)</Radio>
								</RadioGroup>
							</div>
						) : null}
					</div>

					{/* Theme Preview */}
					<div className="mt-6 pt-6 border-t border-neutral-200">
						<div className="flex items-center gap-4">
							<div className="flex-1">
								<h3 className="text-sm font-medium text-neutral-900 mb-2">
									Preview
								</h3>
								<p className="text-xs text-neutral-600">
									Current theme:{" "}
									<span className="font-medium">
										{interfaceTheme === "system"
											? "System"
											: interfaceTheme === "light"
												? themeLight
												: themeDark}
									</span>
								</p>
							</div>
							<div className="flex gap-2">
								{/* Preview cards */}
								<div
									className="w-16 h-16 rounded-lg border-2 border-neutral-300"
									style={{
										backgroundColor:
											interfaceTheme === "light" || interfaceTheme === "system"
												? themeLight === "light"
													? "#fafafa"
													: "#fffbeb"
												: themeDark === "dark"
													? "#0a0a0a"
													: "#0f0f0f",
									}}
								/>
								<div
									className="w-16 h-16 rounded-lg border-2 border-neutral-300"
									style={{
										backgroundColor:
											interfaceTheme === "light" || interfaceTheme === "system"
												? "#ffffff"
												: "#171717",
									}}
								/>
							</div>
						</div>
					</div>
				</section>
			</div>
		</div>
	);
};
