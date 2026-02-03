import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect } from "react";
import {
	type GetAllSettingsQueryResult,
	getGetAllSettingsQueryKey,
	useGetAllSettings,
	useSetSetting,
} from "~/aether-sdk";

export enum ThemeMode {
	LIGHT = "light",
	DARK = "dark",
	SYSTEM = "system",
}

export type LightTheme = "classic" | "amber";
export type DarkTheme = "classic" | "lime";

const SETTING_KEYS = {
	interfaceTheme: "interface.theme",
	themeLight: "theme.light",
	themeDark: "theme.dark",
} as const;

const STORAGE_KEY = "aether-theme";

function getSystemIsDark(): boolean {
	return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

function applyTheme(mode: ThemeMode, light: LightTheme, dark: DarkTheme) {
	const isDark =
		mode === ThemeMode.SYSTEM ? getSystemIsDark() : mode === ThemeMode.DARK;
	const theme = isDark ? `dark-${dark}` : `light-${light}`;
	document.documentElement.setAttribute("data-theme", theme);
	localStorage.setItem(STORAGE_KEY, theme);
}

export function useTheme() {
	const queryClient = useQueryClient();
	const { mutate: setSetting } = useSetSetting();
	const settingsQueryKey = getGetAllSettingsQueryKey();

	const update = useCallback(
		(key: string, value: string) => {
			setSetting(
				{ data: { key, value } },
				{
					onSuccess: () => {
						queryClient.invalidateQueries({ queryKey: settingsQueryKey });
					},
				},
			);
		},
		[queryClient, settingsQueryKey, setSetting],
	);

	const { data } = useGetAllSettings({
		query: {
			select: (r: GetAllSettingsQueryResult) => ({
				mode:
					(r.data?.[SETTING_KEYS.interfaceTheme] as ThemeMode | undefined) ??
					ThemeMode.LIGHT,
				light:
					(r.data?.[SETTING_KEYS.themeLight] as LightTheme | undefined) ??
					"classic",
				dark:
					(r.data?.[SETTING_KEYS.themeDark] as DarkTheme | undefined) ?? "lime",
			}),
		},
	});

	const { mode, light, dark } = data ?? {
		mode: ThemeMode.SYSTEM,
		light: "classic" as LightTheme,
		dark: "classic" as DarkTheme,
	};

	// Listen for system changes
	useEffect(() => {
		if (mode !== ThemeMode.SYSTEM) return;
		const mq = window.matchMedia("(prefers-color-scheme: dark)");
		const handler = () => applyTheme(mode, light, dark);
		mq.addEventListener("change", handler);
		return () => mq.removeEventListener("change", handler);
	}, [mode, light, dark]);

	const updateInterfaceTheme = useCallback(
		(value: ThemeMode) => {
			applyTheme(value, light, dark);
			update(SETTING_KEYS.interfaceTheme, value);
		},
		[light, dark, update],
	);

	const updateColorScheme = useCallback(
		(theme: "light" | "dark", value: LightTheme | DarkTheme) => {
			const newLight = theme === "light" ? (value as LightTheme) : light;
			const newDark = theme === "dark" ? (value as DarkTheme) : dark;
			applyTheme(mode, newLight, newDark);
			update(
				theme === "light" ? SETTING_KEYS.themeLight : SETTING_KEYS.themeDark,
				value as string,
			);
		},
		[mode, light, dark, update],
	);

	return {
		interfaceTheme: mode,
		lightTheme: light,
		darkTheme: dark,
		updateInterfaceTheme,
		updateColorScheme,
	};
}
