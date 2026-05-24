import NavigationAnchor from "@/layouts/NavigationAnchor";
import type { ReactElement } from "react";

const PrimaryNavigation = (): ReactElement => (
  <nav className="col-span-12 flex items-center gap-1 rounded-lg border border-zinc-200 bg-zinc-50 p-1 text-sm font-medium sm:col-span-7 lg:col-span-6">
    <NavigationAnchor href="/" label="Overview" />
    <NavigationAnchor href="/datasets" label="Datasets" />
    <NavigationAnchor href="/genes" label="Genes" />
    <NavigationAnchor href="/analysis" label="Analysis" />
  </nav>
);

export default PrimaryNavigation;
