import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactElement } from "react";
import AppRouter from "@/app/AppRouter";

const queryClient = new QueryClient();

const QueryProvider = (): ReactElement => (
  <QueryClientProvider client={queryClient}>
    <AppRouter />
  </QueryClientProvider>
);

export default QueryProvider;
