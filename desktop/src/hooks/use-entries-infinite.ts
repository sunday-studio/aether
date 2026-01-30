import { useCallback, useMemo, useRef } from "react";
import { useGetEntriesInfinite } from "~/aether-sdk";
import type { Entry } from "~/aether-sdk/models";

interface UseEntriesInfiniteResult {
	items: Entry[];
	hasMore: boolean;
	nextCursor: string | null;
	isFetchingMore: boolean;
	isLoading: boolean;
	sentinelRef: (node: HTMLElement | null) => void;
	fetchMore: () => void;
}

export function useEntriesInfinite(): UseEntriesInfiniteResult {
	const { data, fetchNextPage, hasNextPage, isFetchingNextPage, isLoading } =
		useGetEntriesInfinite(
			{},
			{
				query: {
					getNextPageParam: (lastPage) =>
						lastPage.data?.nextCursor ?? undefined,
				},
			},
		);

	const observerRef = useRef<IntersectionObserver | null>(null);

	const items = useMemo(() => {
		return data?.pages.flatMap((page) => page.data?.items ?? []) ?? [];
	}, [data?.pages]);

	const lastPage = data?.pages[data.pages.length - 1];
	const hasMore = lastPage?.data?.hasMore ?? false;
	const nextCursor = lastPage?.data?.nextCursor ?? null;

	const fetchMore = useCallback(() => {
		if (hasNextPage && !isFetchingNextPage) {
			fetchNextPage();
		}
	}, [hasNextPage, isFetchingNextPage, fetchNextPage]);

	const sentinelRef = useCallback(
		(node: HTMLElement | null) => {
			if (observerRef.current) {
				observerRef.current.disconnect();
			}

			if (isFetchingNextPage || !hasNextPage) {
				return;
			}

			observerRef.current = new IntersectionObserver(
				(entries) => {
					if (
						entries[0]?.isIntersecting &&
						hasNextPage &&
						!isFetchingNextPage
					) {
						fetchNextPage();
					}
				},
				{ rootMargin: "100px" },
			);

			if (node) {
				observerRef.current.observe(node);
			}
		},
		[fetchNextPage, hasNextPage, isFetchingNextPage],
	);

	return {
		items,
		hasMore,
		nextCursor,
		isFetchingMore: isFetchingNextPage,
		isLoading,
		sentinelRef,
		fetchMore,
	};
}
