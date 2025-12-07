import { QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { Suspense } from "react";
import { Toaster } from "sonner";
import { Entries } from "./features/entries/entries";
import { initQueryClient } from "./utils/query-client";

import "./app.css";

const queryClient = initQueryClient();

function App() {
	return (
		<Suspense fallback={<div>Loading...</div>}>
			<QueryClientProvider client={queryClient}>
				<Toaster />
				<ReactQueryDevtools
					buttonPosition="bottom-left"
					initialIsOpen={false}
				/>
				<Entries />
			</QueryClientProvider>
		</Suspense>
	);
}

export default App;
