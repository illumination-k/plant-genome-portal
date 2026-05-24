import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { createBrowserRouter, NavLink, Outlet, RouterProvider } from "react-router";
import { Button } from "@base-ui/react/button";
import { QueryClient, QueryClientProvider, useQuery } from "@tanstack/react-query";
import * as v from "valibot";
import "./styles.css";

const queryClient = new QueryClient();

const datasetSchema = v.object({
  assembly: v.string(),
  species: v.string(),
  status: v.string(),
});
const datasetsSchema = v.array(datasetSchema);

type Dataset = v.InferOutput<typeof datasetSchema>;

const datasets = v.parse(datasetsSchema, [
  { assembly: "IRGSP-1.0", species: "Oryza sativa", status: "Ready" },
  {
    assembly: "TAIR10",
    species: "Arabidopsis thaliana",
    status: "Ready",
  },
  {
    assembly: "Zm-B73-REFERENCE",
    species: "Zea mays",
    status: "Indexing",
  },
]);

const routes = createBrowserRouter([
  {
    path: "/",
    element: <RootLayout />,
    children: [
      { index: true, element: <DashboardPage /> },
      { path: "datasets", element: <DatasetsPage /> },
      { path: "analysis", element: <AnalysisPage /> },
    ],
  },
]);

function RootLayout() {
  return (
    <div className="min-h-screen bg-stone-50 text-zinc-950">
      <header className="border-b border-zinc-200 bg-white">
        <div className="mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-4 px-5 py-4">
          <div>
            <p className="text-xs font-semibold uppercase tracking-wide text-emerald-700">
              Plant Genome Portal
            </p>
            <h1 className="text-xl font-semibold text-zinc-950">Genome workspace</h1>
          </div>
          <nav className="flex items-center gap-1 rounded-lg border border-zinc-200 bg-zinc-50 p-1 text-sm font-medium">
            <NavigationLink to="/">Overview</NavigationLink>
            <NavigationLink to="/datasets">Datasets</NavigationLink>
            <NavigationLink to="/analysis">Analysis</NavigationLink>
          </nav>
        </div>
      </header>
      <main className="mx-auto max-w-6xl px-5 py-8">
        <Outlet />
      </main>
    </div>
  );
}

function NavigationLink(props: { to: string; children: string }) {
  return (
    <NavLink
      className={({ isActive }) =>
        [
          "rounded-md px-3 py-2 transition",
          isActive
            ? "bg-white text-emerald-800 shadow-sm"
            : "text-zinc-600 hover:bg-white hover:text-zinc-950",
        ].join(" ")
      }
      to={props.to}
    >
      {props.children}
    </NavLink>
  );
}

function DashboardPage() {
  return (
    <section className="grid gap-6 lg:grid-cols-[1.4fr_0.8fr]">
      <div className="rounded-lg border border-zinc-200 bg-white p-6">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <h2 className="text-2xl font-semibold">Reference genomes</h2>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-zinc-600">
              Search, compare, and inspect plant genome assemblies from a React frontend backed by
              Vite, React Router, Tailwind CSS, and Base UI.
            </p>
          </div>
          <Button className="rounded-md bg-emerald-700 px-4 py-2 text-sm font-semibold text-white transition hover:bg-emerald-800 focus:outline-none focus:ring-2 focus:ring-emerald-600 focus:ring-offset-2">
            New query
          </Button>
        </div>
        <div className="mt-6 grid gap-4 sm:grid-cols-3">
          <Metric label="Assemblies" value="128" />
          <Metric label="Species" value="42" />
          <Metric label="Annotations" value="316k" />
        </div>
      </div>
      <div className="rounded-lg border border-zinc-200 bg-white p-6">
        <h2 className="text-base font-semibold">Active pipeline</h2>
        <div className="mt-4 space-y-4">
          {["Import FASTA", "Index features", "Publish dataset"].map((step, index) => (
            <div className="flex items-center gap-3" key={step}>
              <div className="grid size-8 place-items-center rounded-full bg-emerald-100 text-sm font-semibold text-emerald-800">
                {index + 1}
              </div>
              <span className="text-sm text-zinc-700">{step}</span>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function DatasetsPage() {
  const { data = [] } = useDatasets();

  return (
    <section className="rounded-lg border border-zinc-200 bg-white p-6">
      <h2 className="text-2xl font-semibold">Datasets</h2>
      <div className="mt-5 overflow-hidden rounded-lg border border-zinc-200">
        <table className="w-full text-left text-sm">
          <thead className="bg-zinc-50 text-zinc-600">
            <tr>
              <th className="px-4 py-3 font-medium">Species</th>
              <th className="px-4 py-3 font-medium">Assembly</th>
              <th className="px-4 py-3 font-medium">Status</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-zinc-200">
            {data.map((dataset) => (
              <DatasetRow key={dataset.assembly} {...dataset} />
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function AnalysisPage() {
  return (
    <section className="rounded-lg border border-zinc-200 bg-white p-6">
      <h2 className="text-2xl font-semibold">Analysis</h2>
      <p className="mt-2 text-sm leading-6 text-zinc-600">
        Route-level screens are ready for genome search, annotation comparison, and downstream
        visualization features.
      </p>
    </section>
  );
}

function Metric(props: { label: string; value: string }) {
  return (
    <div className="rounded-lg border border-zinc-200 bg-zinc-50 p-4">
      <p className="text-sm text-zinc-600">{props.label}</p>
      <p className="mt-2 text-2xl font-semibold text-zinc-950">{props.value}</p>
    </div>
  );
}

function useDatasets() {
  return useQuery({
    queryKey: ["datasets"],
    queryFn: async () => datasets,
    staleTime: 60_000,
  });
}

function DatasetRow(props: Dataset) {
  return (
    <tr>
      <td className="px-4 py-3 font-medium text-zinc-900">{props.species}</td>
      <td className="px-4 py-3 text-zinc-600">{props.assembly}</td>
      <td className="px-4 py-3">
        <span className="rounded-md bg-sky-100 px-2 py-1 text-xs font-semibold text-sky-800">
          {props.status}
        </span>
      </td>
    </tr>
  );
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={routes} />
    </QueryClientProvider>
  </StrictMode>,
);
