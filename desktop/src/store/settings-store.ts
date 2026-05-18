import { type QueryClient, useQueryClient } from '@tanstack/react-query';
import { useCallback, useEffect } from 'react';
import { proxy, useSnapshot } from 'valtio';
import {
	getGetAllSettingsQueryKey,
	setSetting,
	useGetAllSettings,
	type getAllSettingsResponse,
} from '~/aether-sdk';

type SettingsMap = Record<string, string>;
type SettingsUpdate = Record<string, string>;

type SettingsStoreState = {
	values: SettingsMap;
	isHydrated: boolean;
};

const settingsStore = proxy<SettingsStoreState>({
	values: {},
	isHydrated: false,
});

const settingsQueryKey = getGetAllSettingsQueryKey();

function toSettingsResponse(values: SettingsMap): getAllSettingsResponse {
	return {
		data: values,
		status: 200,
		headers: new Headers(),
	};
}

function setSettingsValues(values: SettingsMap) {
	settingsStore.values = values;
	settingsStore.isHydrated = true;
}

function mergeSettingsValues(values: SettingsUpdate) {
	settingsStore.values = {
		...settingsStore.values,
		...values,
	};
	settingsStore.isHydrated = true;
}

async function persistSettings(queryClient: QueryClient, nextValues: SettingsUpdate) {
	const previousValues = settingsStore.values;
	const mergedValues = {
		...previousValues,
		...nextValues,
	};

	mergeSettingsValues(nextValues);
	queryClient.setQueryData(settingsQueryKey, toSettingsResponse(mergedValues));

	try {
		await Promise.all(Object.entries(nextValues).map(([key, value]) => setSetting({ key, value })));
		await queryClient.invalidateQueries({ queryKey: settingsQueryKey });
	} catch (error) {
		setSettingsValues(previousValues);
		queryClient.setQueryData(settingsQueryKey, toSettingsResponse(previousValues));
		throw error;
	}
}

export function useSettingsStore() {
	const queryClient = useQueryClient();
	const { data, isLoading, isFetching, refetch } = useGetAllSettings({
		query: {
			queryKey: settingsQueryKey,
		},
	});
	const snapshot = useSnapshot(settingsStore);

	useEffect(() => {
		if (data?.data) {
			setSettingsValues(data.data);
		}
	}, [data]);

	const setValue = useCallback(
		async (key: string, value: string) => {
			await persistSettings(queryClient, { [key]: value });
		},
		[queryClient],
	);

	const setValues = useCallback(
		async (values: SettingsUpdate) => {
			await persistSettings(queryClient, values);
		},
		[queryClient],
	);

	const getValue = useCallback(
		(key: string, fallback?: string) => snapshot.values[key] ?? fallback,
		[snapshot.values],
	);

	return {
		settings: snapshot.values,
		isHydrated: snapshot.isHydrated,
		isLoading,
		isFetching,
		getValue,
		setValue,
		setValues,
		refetch,
	};
}

export function useSetting(key: string, fallback?: string) {
	const { getValue, setValue, ...rest } = useSettingsStore();

	const updateValue = useCallback(
		async (value: string) => {
			await setValue(key, value);
		},
		[key, setValue],
	);

	return {
		value: getValue(key, fallback),
		setValue: updateValue,
		...rest,
	};
}
