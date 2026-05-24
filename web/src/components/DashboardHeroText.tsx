import type { ReactElement } from "react";

const DashboardHeroText = (): ReactElement => (
  <div>
    <h2 className="text-2xl font-semibold">Reference genomes</h2>
    <p className="mt-2 max-w-2xl text-sm leading-6 text-zinc-600">
      Search, compare, and inspect plant genome assemblies from a React frontend backed by Vite,
      React Router, Tailwind CSS, and Base UI.
    </p>
  </div>
);

export default DashboardHeroText;
