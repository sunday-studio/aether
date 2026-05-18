import type { QueryClient } from '@tanstack/react-query';

export function invalidateSearchQueries(queryClient: QueryClient) {
	queryClient.invalidateQueries({ queryKey: ['command-palette-search'] });
	queryClient.invalidateQueries({ queryKey: ['searchLinkableResources'] });
}
