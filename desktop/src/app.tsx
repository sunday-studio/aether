import {Suspense} from "react";
import {Entries} from "./features/entries/entries";
import {QueryClientProvider} from "@tanstack/react-query";
import {initQueryClient} from "./services/query-client";
import {ReactQueryDevtools} from "@tanstack/react-query-devtools";
import "./app.css";

const queryClient = initQueryClient();

function App() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <QueryClientProvider client={queryClient}>
        <ReactQueryDevtools buttonPosition="bottom-left" initialIsOpen={false} />

        <Entries />
      </QueryClientProvider>
    </Suspense>
  );
}

export default App;
