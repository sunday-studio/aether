import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo } from "react";
import { useGetAllSettings } from "~/aether-sdk";
import customFetch from "~/lib/api-client";

type ThemeMode = "light" | "dark" | "system";
type LightTheme = "light" | "amber";
type DarkTheme = "dark" | "lime";

interface ThemeSettings {
	interfaceTheme: ThemeMode;
	themeLight: LightTheme;
	themeDark: DarkTheme;
}

const SETTING_KEYS = {
	interfaceTheme: "interface.theme",
	themeLight: "theme.light",
	themeDark: "theme.dark",
} as const;

const DEFAULT_SETTINGS: ThemeSettings = {
	interfaceTheme: "system",
	themeLight: "light",
	themeDark: "dark",
};

// Fetch a setting value
async function fetchSetting(key: string): Promise<string | null> {
	try {
		const response = await customFetch<{
			data: { key: string; value: string | null };
		}>(`/v1/settings?key=${encodeURIComponent(key)}`, { method: "GET" });
		// The Tauri command returns { key, value } directly, which is wrapped in { data: ... }
		const result = response.data as { key: string; value: string | null };
		return result?.value ?? null;
	} catch (error) {
		console.error(`Failed to fetch setting ${key}:`, error);
		return null;
	}
}

// Set a setting value
async function updateSetting(key: string, value: string): Promise<void> {
	try {
		await customFetch(`/v1/settings`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ key, value }),
		});
	} catch (error) {
		console.error(`Failed to update setting ${key}:`, error);
		throw error;
	}
}

// Detect system color scheme preference
function getSystemPreference(): "light" | "dark" {
	if (typeof window === "undefined") return "light";
	return window.matchMedia("(prefers-color-scheme: dark)").matches
		? "dark"
		: "light";
}

// Get the effective theme based on mode and system preference
function getEffectiveTheme(
	mode: ThemeMode,
	lightTheme: LightTheme,
	darkTheme: DarkTheme,
): "light" | "amber" | "dark" | "lime" {
	if (mode === "system") {
		const systemPref = getSystemPreference();
		return systemPref === "dark" ? darkTheme : lightTheme;
	}
	return mode === "dark" ? darkTheme : lightTheme;
}

export function useTheme() {
	const queryClient = useQueryClient();
	const { data } = useGetAllSettings();
	console.log(data);

	// Fetch all theme settings
	const { data: interfaceTheme } = useQuery({
		queryKey: ["settings", SETTING_KEYS.interfaceTheme],
		queryFn: () => fetchSetting(SETTING_KEYS.interfaceTheme),
		placeholderData: DEFAULT_SETTINGS.interfaceTheme,
	});

	const { data: themeLight } = useQuery({
		queryKey: ["settings", SETTING_KEYS.themeLight],
		queryFn: () => fetchSetting(SETTING_KEYS.themeLight),
		placeholderData: DEFAULT_SETTINGS.themeLight,
	});

	const { data: themeDark } = useQuery({
		queryKey: ["settings", SETTING_KEYS.themeDark],
		queryFn: () => fetchSetting(SETTING_KEYS.themeDark),
		placeholderData: DEFAULT_SETTINGS.themeDark,
	});

	// Mutations for updating settings
	const setInterfaceTheme = useMutation({
		mutationFn: (value: ThemeMode) =>
			updateSetting(SETTING_KEYS.interfaceTheme, value),
		onSuccess: () => {
			queryClient.invalidateQueries({
				queryKey: ["settings", SETTING_KEYS.interfaceTheme],
			});
		},
	});

	const setThemeLight = useMutation({
		mutationFn: (value: LightTheme) =>
			updateSetting(SETTING_KEYS.themeLight, value),
		onSuccess: () => {
			queryClient.invalidateQueries({
				queryKey: ["settings", SETTING_KEYS.themeLight],
			});
		},
	});

	const setThemeDark = useMutation({
		mutationFn: (value: DarkTheme) =>
			updateSetting(SETTING_KEYS.themeDark, value),
		onSuccess: () => {
			queryClient.invalidateQueries({
				queryKey: ["settings", SETTING_KEYS.themeDark],
			});
		},
	});

	// Compute effective theme
	const effectiveTheme = useMemo(() => {
		const mode = (interfaceTheme ??
			DEFAULT_SETTINGS.interfaceTheme) as ThemeMode;
		const light = (themeLight ?? DEFAULT_SETTINGS.themeLight) as LightTheme;
		const dark = (themeDark ?? DEFAULT_SETTINGS.themeDark) as DarkTheme;
		return getEffectiveTheme(mode, light, dark);
	}, [interfaceTheme, themeLight, themeDark]);

	// Apply theme to document
	useEffect(() => {
		if (typeof document === "undefined") return;
		const root = document.documentElement;
		root.setAttribute("data-theme", effectiveTheme);
	}, [effectiveTheme]);

	// Listen to system preference changes when in system mode
	useEffect(() => {
		if (typeof window === "undefined") return;
		const mode = (interfaceTheme ??
			DEFAULT_SETTINGS.interfaceTheme) as ThemeMode;
		if (mode !== "system") return;

		const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
		const handleChange = () => {
			// Force re-computation by invalidating queries
			queryClient.invalidateQueries({ queryKey: ["settings"] });
		};

		mediaQuery.addEventListener("change", handleChange);
		return () => mediaQuery.removeEventListener("change", handleChange);
	}, [interfaceTheme, queryClient]);

	return {
		// Current values
		interfaceTheme: (interfaceTheme ??
			DEFAULT_SETTINGS.interfaceTheme) as ThemeMode,
		themeLight: (themeLight ?? DEFAULT_SETTINGS.themeLight) as LightTheme,
		themeDark: (themeDark ?? DEFAULT_SETTINGS.themeDark) as DarkTheme,
		effectiveTheme,
		// Setters
		setInterfaceTheme: (value: ThemeMode) => setInterfaceTheme.mutate(value),
		setThemeLight: (value: LightTheme) => setThemeLight.mutate(value),
		setThemeDark: (value: DarkTheme) => setThemeDark.mutate(value),
		// Loading states
		isLoading:
			setInterfaceTheme.isPending ||
			setThemeLight.isPending ||
			setThemeDark.isPending,
	};
}
