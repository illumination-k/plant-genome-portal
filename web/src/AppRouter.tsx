import { RouterProvider, createBrowserRouter } from "react-router";
import type { ReactElement } from "react";
import RootLayout from "@/layouts/RootLayout";
import AnalysisPage from "@/pages/AnalysisPage";
import DashboardPage from "@/pages/DashboardPage";
import DatasetsPage from "@/pages/DatasetsPage";
import GeneDetailPage from "@/pages/GeneDetailPage";
import GenesPage from "@/pages/GenesPage";

const routes = createBrowserRouter([
  {
    children: [
      { element: <DashboardPage />, index: true },
      { element: <DatasetsPage />, path: "datasets" },
      { element: <GenesPage />, path: "genes" },
      { element: <GeneDetailPage />, path: "genes/:geneId" },
      { element: <AnalysisPage />, path: "analysis" },
    ],
    element: <RootLayout />,
    path: "/",
  },
]);

const AppRouter = (): ReactElement => <RouterProvider router={routes} />;

export default AppRouter;
