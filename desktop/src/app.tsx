import { QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { Suspense } from "react";
import { RouterProvider } from "react-router";
import { Toaster } from "sonner";
import { ThemeProvider } from "./context/theme-context";
import { router } from "./features/router";
import { useSyncDataRefresh } from "./hooks/use-sync-data-refresh";
import { initQueryClient } from "./utils/query-client";

import "./app.css";

const queryClient = initQueryClient();

function SyncDataRefresh({ children }: { children: React.ReactNode }) {
	// useSyncDataRefresh();
	return <>{children}</>;
}

function App() {
	return (
		<Suspense fallback={<div>Loading...</div>}>
			<QueryClientProvider client={queryClient}>
				<SyncDataRefresh>
					<ThemeProvider>
						<Toaster />
						<ReactQueryDevtools
							buttonPosition="top-right"
							initialIsOpen={false}
						/>
						<RouterProvider router={router} />
					</ThemeProvider>
				</SyncDataRefresh>
			</QueryClientProvider>
		</Suspense>
	);
}

export default App;
