import { useCallback, useMemo, useRef } from "react";

interface UseInfiniteScrollOptions<TPage, TItem> {
	/** Array of pages from useInfiniteQuery */
	pages: TPage[] | undefined;
	/** Function to extract items from each page */
	getItems: (page: TPage) => TItem[];
	/** Function to fetch the next page */
	fetchNextPage: () => void;
	/** Whether there are more pages to fetch */
	hasNextPage: boolean | undefined;
	/** Whether we're currently fetching the next page */
	isFetchingNextPage: boolean;
	/** IntersectionObserver threshold (0-1), default 0 */
	threshold?: number;
	/** Root margin for IntersectionObserver, default "100px" */
	rootMargin?: string;
}

interface UseInfiniteScrollResult<TItem> {
	/** Flattened array of all items from all pages */
	items: TItem[];
	/** Ref callback to attach to sentinel element */
	sentinelRef: (node: HTMLElement | null) => void;
	/** Whether we're currently fetching more items */
	isFetchingMore: boolean;
	/** Whether there are more items to fetch */
	hasMore: boolean;
	/** Manually trigger fetch (useful for virtualized lists) */
	fetchMore: () => void;
}

/**
 * Hook for infinite scroll functionality.
 *
 * Provides a sentinel ref that triggers fetching when it becomes visible,
 * and a flattened array of all items from all pages.
 *
 * @example
 * ```tsx
 * const { items, sentinelRef, isFetchingMore, hasMore } = useInfiniteScroll({
 *   pages: data?.pages,
 *   getItems: (page) => page.data?.items ?? [],
 *   fetchNextPage,
 *   hasNextPage,
 *   isFetchingNextPage,
 * });
 *
 * return (
 *   <div>
 *     {items.map(item => <Item key={item.id} item={item} />)}
 *     <div ref={sentinelRef}>
 *       {isFetchingMore && <Loader />}
 *     </div>
 *   </div>
 * );
 * ```
 */
export function useInfiniteScroll<TPage, TItem>({
	pages,
	getItems,
	fetchNextPage,
	hasNextPage,
	isFetchingNextPage,
	threshold = 0,
	rootMargin = "100px",
}: UseInfiniteScrollOptions<TPage, TItem>): UseInfiniteScrollResult<TItem> {
	const observerRef = useRef<IntersectionObserver | null>(null);

	// Flatten all pages into a single array of items
	const items = useMemo(() => {
		return pages?.flatMap(getItems) ?? [];
	}, [pages, getItems]);

	// Manual fetch function (useful for virtualized lists)
	const fetchMore = useCallback(() => {
		if (hasNextPage && !isFetchingNextPage) {
			fetchNextPage();
		}
	}, [hasNextPage, isFetchingNextPage, fetchNextPage]);

	// Sentinel ref callback that sets up the IntersectionObserver
	const sentinelRef = useCallback(
		(node: HTMLElement | null) => {
			// Disconnect previous observer
			if (observerRef.current) {
				observerRef.current.disconnect();
			}

			// Don't observe if we're already fetching or there's nothing more to fetch
			if (isFetchingNextPage || !hasNextPage) {
				return;
			}

			// Create new observer
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
				{
					threshold,
					rootMargin,
				},
			);

			// Observe the sentinel element
			if (node) {
				observerRef.current.observe(node);
			}
		},
		[fetchNextPage, hasNextPage, isFetchingNextPage, threshold, rootMargin],
	);

	return {
		items,
		sentinelRef,
		isFetchingMore: isFetchingNextPage,
		hasMore: hasNextPage ?? false,
		fetchMore,
	};
}
