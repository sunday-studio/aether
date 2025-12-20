import { QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { Suspense } from "react";
import { RouterProvider } from "react-router";
import { Toaster } from "sonner";
import { router } from "./features/router";
import { initQueryClient } from "./utils/query-client";

import "./app.css";

const queryClient = initQueryClient();

function App() {
	return (
		<Suspense fallback={<div>Loading...</div>}>
			<QueryClientProvider client={queryClient}>
				<Toaster />
				<ReactQueryDevtools buttonPosition="top-right" initialIsOpen={false} />
				<RouterProvider router={router} />
			</QueryClientProvider>
		</Suspense>
	);
}

export default App;
