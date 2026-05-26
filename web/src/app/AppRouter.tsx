import { RouterProvider, createBrowserRouter } from "react-router";
import { Suspense, lazy, type ReactElement } from "react";
import RootLayout from "@/app/layouts/RootLayout";
import AnalysisPage from "@/features/analysis/pages/AnalysisPage";
import DashboardPage from "@/features/landing/pages/DashboardPage";
import DatasetsPage from "@/features/datasets/pages/DatasetsPage";
import GenesPage from "@/features/genes/pages/GenesPage";

const BlastPage = lazy(() => import("@/features/tools/blast/pages/BlastPage"));
const BrowserPage = lazy(() => import("@/features/genome-browser/pages/BrowserPage"));
const FetchPage = lazy(() => import("@/features/tools/fetch/pages/FetchPage"));
const GeneDetailPage = lazy(() => import("@/features/genes/pages/GeneDetailPage"));
const KeggPathwayPage = lazy(() => import("@/features/kegg/pages/KeggPathwayPage"));

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
      { element: lazyPage(<KeggPathwayPage />), path: "kegg/pathway/:pathwayId" },
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
