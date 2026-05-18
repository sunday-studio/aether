import type { QueryClient } from '@tanstack/react-query';
import { getGetEntriesInfiniteQueryKey, getGetEntriesQueryKey } from '~/aether-sdk';
import { invalidateSearchQueries } from '~/lib/search-query-invalidation';

/**
 * Invalidate both entry query keys (infinite and non-infinite) so timeline and grid stay in sync.
 */
export function invalidateEntryQueries(queryClient: QueryClient) {
	queryClient.invalidateQueries({ queryKey: getGetEntriesInfiniteQueryKey() });
	queryClient.invalidateQueries({ queryKey: getGetEntriesQueryKey() });
	invalidateSearchQueries(queryClient);
}
