import { useCallback, useMemo, useRef } from "react";
import { useGetEntriesInfinite } from "~/aether-sdk";
import type { Entry } from "~/aether-sdk/models";

interface UseEntriesInfiniteResult {
	/** Flattened array of all entries from all pages */
	items: Entry[];
	/** Whether there are more pages to fetch */
	hasMore: boolean;
	/** Cursor for the next page */
	nextCursor: string | null;
	/** Whether we're currently fetching the next page */
	isFetchingMore: boolean;
	/** Whether the initial load is happening */
	isLoading: boolean;
	/** Ref callback to attach to sentinel element for auto-fetching */
	sentinelRef: (node: HTMLElement | null) => void;
	/** Manually trigger fetching next page */
	fetchMore: () => void;
}

/**
 * Hook for infinite scroll entries with a flat, easy-to-use interface.
 * 
 * @example
 * ```tsx
 * const { items, sentinelRef, isFetchingMore, hasMore } = useEntriesInfinite();
 * 
 * return (
 *   <div>
 *     {items.map(entry => <Entry key={entry.id} entry={entry} />)}
 *     <div ref={sentinelRef}>
 *       {isFetchingMore && <Loader />}
 *     </div>
 *   </div>
 * );
 * ```
 */
export function useEntriesInfinite(): UseEntriesInfiniteResult {
	const {
		data,
		fetchNextPage,
		hasNextPage,
		isFetchingNextPage,
		isLoading,
	} = useGetEntriesInfinite(
		{},
		{
			query: {
				getNextPageParam: (lastPage) => lastPage.data?.nextCursor ?? undefined,
			},
		},
	);

	const observerRef = useRef<IntersectionObserver | null>(null);

	// Flatten all pages into a single array
	const items = useMemo(() => {
		return data?.pages.flatMap((page) => page.data?.items ?? []) ?? [];
	}, [data?.pages]);

	// Get pagination info from the last page
	const lastPage = data?.pages[data.pages.length - 1];
	const hasMore = lastPage?.data?.hasMore ?? false;
	const nextCursor = lastPage?.data?.nextCursor ?? null;

	// Manual fetch function
	const fetchMore = useCallback(() => {
		if (hasNextPage && !isFetchingNextPage) {
			fetchNextPage();
		}
	}, [hasNextPage, isFetchingNextPage, fetchNextPage]);

	// Sentinel ref callback for intersection observer
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
					if (entries[0]?.isIntersecting && hasNextPage && !isFetchingNextPage) {
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
