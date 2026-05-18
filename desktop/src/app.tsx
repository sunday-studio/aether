import { QueryClientProvider } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import { Suspense } from 'react';
import { RouterProvider } from 'react-router';
import { Toaster } from 'sonner';
import { ThemeProvider } from './context/theme-context';
import { UpdaterProvider } from './context/updater-context';
import { OnboardingGate } from './features/onboarding/onboarding-gate';
import { router } from './features/router';
import { SearchEmbeddingNotificationListener } from './components/shared/search-embedding-notification';
import { initQueryClient } from './utils/query-client';

import './app.css';

const queryClient = initQueryClient();

// function SyncDataRefresh({ children }: { children: React.ReactNode }) {
// 	useSyncDataRefresh();
// 	return <>{children}</>;
// }

// function UpdateListener({ children }: { children: React.ReactNode }) {
// 	return (
// 		<>
// 			<UpdateNotificationListener />
// 			{children}
// 		</>
// 	);
// }

function App() {
	return (
		<Suspense fallback={<div>Loading...</div>}>
			<QueryClientProvider client={queryClient}>
				<ThemeProvider>
					<UpdaterProvider>
						<Toaster />
						<SearchEmbeddingNotificationListener />
						<ReactQueryDevtools buttonPosition='top-right' initialIsOpen={false} />
						<OnboardingGate>
							<RouterProvider router={router} />
						</OnboardingGate>
					</UpdaterProvider>
				</ThemeProvider>
			</QueryClientProvider>
		</Suspense>
	);
}

export default App;
