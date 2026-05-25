import { RouterProvider, createBrowserRouter } from "react-router";
import { Suspense, lazy, type ReactElement } from "react";
import RootLayout from "@/layouts/RootLayout";
import AnalysisPage from "@/pages/AnalysisPage";
import DashboardPage from "@/pages/DashboardPage";
import DatasetsPage from "@/pages/DatasetsPage";
import GenesPage from "@/pages/GenesPage";

const BlastPage = lazy(() => import("@/pages/BlastPage"));
const BrowserPage = lazy(() => import("@/pages/BrowserPage"));
const FetchPage = lazy(() => import("@/pages/FetchPage"));
const GeneDetailPage = lazy(() => import("@/pages/GeneDetailPage"));

const lazyPage = (page: ReactElement): ReactElement => (
  <Suspense fallback={undefined}>{page}</Suspense>
);

const routes = createBrowserRouter([
  {
    children: [
      { element: <DashboardPage />, index: true },
      { element: lazyPage(<BrowserPage />), path: "browser" },
      { element: <DatasetsPage />, path: "datasets" },
      { element: <GenesPage />, path: "genes" },
      { element: lazyPage(<GeneDetailPage />), path: "genes/:geneId" },
      { element: lazyPage(<FetchPage />), path: "tools/fetch" },
      { element: lazyPage(<BlastPage />), path: "tools/blast" },
      { element: <AnalysisPage />, path: "analysis" },
    ],
    element: <RootLayout />,
    path: "/",
  },
]);

const AppRouter = (): ReactElement => <RouterProvider router={routes} />;

export default AppRouter;
