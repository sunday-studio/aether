import { tv } from 'tailwind-variants';
import { Select, SelectItem } from '~/components/shared/select';
import { type DarkTheme, type LightTheme, type ThemeMode, useTheme } from '~/hooks/use-theme';

const containerStyles = tv({
	base: 'flex items-center justify-between gap-4',
	variants: {
		isDisabled: {
			true: 'cursor-not-allowed opacity-30',
			false: 'bg-transparent',
		},
	},
});

export const PreferencesSection = () => {
	const { updateInterfaceTheme, updateColorScheme, interfaceTheme, lightTheme, darkTheme } =
		useTheme();

	const isLightMode = interfaceTheme === 'light';
	const isDarkMode = interfaceTheme === 'dark';

	return (
		<div className='w-full space-y-10'>
			<div>
				<h3 className='text-lg font-medium'>Preferences</h3>
				<p className='text-sm text-(--color-secondary-text)'>Customize your preferences here.</p>
			</div>

			<div className='flex items-center justify-between gap-4'>
				<div>
					<h4 className='text-md font-medium'>Interface mode</h4>
					<p className='text-sm text-(--color-secondary-text)'>
						Select your preferred interface color mode.
					</p>
				</div>

				<div>
					<Select
						placeholder='System preference'
						value={interfaceTheme}
						onChange={value => updateInterfaceTheme(value as ThemeMode)}
						items={[
							{ label: 'Light', value: 'light' },
							{ label: 'Dark', value: 'dark' },
							{ label: 'System', value: 'system' },
						]}
					>
						<SelectItem id='light'>Light</SelectItem>
						<SelectItem id='dark'>Dark</SelectItem>
						<SelectItem id='system'>System</SelectItem>
					</Select>
				</div>
			</div>

			<div className={containerStyles({ isDisabled: isDarkMode })}>
				<div>
					<h4 className='text-md font-medium'>Light mode</h4>
					<p className='text-sm text-(--color-secondary-text)'>
						Select your preferred light theme.
					</p>
				</div>

				<div>
					<Select
						placeholder='Light mode'
						value={lightTheme}
						onChange={value => updateColorScheme('light', value as LightTheme)}
						isDisabled={isDarkMode}
						items={[
							{ label: 'Classic', value: 'classic' },
							// { label: "Amber", value: "amber" },
						]}
					>
						<SelectItem id='classic'>Classic</SelectItem>
						{/* <SelectItem id="amber">Amber</SelectItem> */}
					</Select>
				</div>
			</div>

			<div className={containerStyles({ isDisabled: isLightMode })}>
				<div>
					<h4 className='text-md font-medium'>Dark mode</h4>
					<p className='text-sm text-(--color-secondary-text)'>Select your preferred dark theme.</p>
				</div>

				<div>
					<Select
						placeholder='Dark mode'
						value={darkTheme}
						onChange={value => updateColorScheme('dark', value as DarkTheme)}
						isDisabled={isLightMode}
						items={[
							// { label: "Classic", value: "classic" },
							{ label: 'Lime', value: 'lime' },
						]}
					>
						{/* <SelectItem id="classic">Classic</SelectItem> */}
						<SelectItem id='lime'>Lime</SelectItem>
					</Select>
				</div>
			</div>
		</div>
	);
};
