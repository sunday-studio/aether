import { useCallback, useEffect, useRef, useState } from "react";

/**
 * @deprecated Use useDebounceCallback instead
 */
export function useDebouncedValue(value: string, delay = 500) {
	const [debouncedValue, setDebouncedValue] = useState(value);

	useEffect(() => {
		const timer = setTimeout(() => {
			setDebouncedValue(value);
		}, delay);

		return () => clearTimeout(timer);
	}, [value, delay]);

	return debouncedValue;
}

export function useDebounceCallback<T extends (...args: any[]) => void>(
	callback: T,
	delay: number,
): T {
	const timeoutRef = useRef<NodeJS.Timeout | null>(null);

	// biome-ignore lint/correctness/useExhaustiveDependencies: Should be fine since we're using a ref
	return useCallback(
		((...args) => {
			if (timeoutRef.current) {
				clearTimeout(timeoutRef.current);
			}
			timeoutRef.current = setTimeout(() => {
				callback(...args);
			}, delay);
		}) as T,
		[callback, delay],
	);
}
