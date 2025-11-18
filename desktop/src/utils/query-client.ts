import {createSyncStoragePersister} from "@tanstack/query-sync-storage-persister";
import {QueryClient} from "@tanstack/react-query";
import {persistQueryClient} from "@tanstack/react-query-persist-client";

export const initQueryClient = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        refetchOnWindowFocus: false,
        gcTime: 1000 * 60 * 60 * 24 * 7, // 7 days

        retry: false,
        // (retryCount) => {
        //   if (retryCount === 2) return false;
        //   return true;
        // },
      },
    },
  });

  const localStoragePersister = createSyncStoragePersister({
    storage: window.localStorage,
  });

  persistQueryClient({
    queryClient,
    dehydrateOptions: {
      shouldDehydrateQuery(query) {
        if (query.meta?.persist === false) return false;
        return query.state.status === "success";
      },
    },
    persister: localStoragePersister,
  });

  return queryClient;
};
