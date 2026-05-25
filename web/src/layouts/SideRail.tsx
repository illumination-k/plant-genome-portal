import type { ReactElement } from "react";
import SideRailGroup from "@/layouts/SideRailGroup";

const exploreItems = [
  { label: "Search", to: "/" },
  { label: "Genes", to: "/genes" },
  { label: "Genome browser", to: "/browser" },
];

const genomeItems = [
  { disabled: true, label: "Species", to: "/species" },
  { label: "Assemblies", to: "/datasets" },
];

const toolItems = [
  { label: "BLAST", to: "/tools/blast" },
  { disabled: true, label: "Region lookup", to: "/tools/region" },
];

const dataItems = [
  { label: "Analysis", to: "/analysis" },
  { disabled: true, label: "Downloads", to: "/downloads" },
  { disabled: true, label: "API reference", to: "/api" },
];

const SideRail = (): ReactElement => (
  <aside
    aria-label="Primary navigation"
    className="sticky top-12 hidden h-[calc(100dvh-3rem)] w-60 shrink-0 overflow-y-auto border-r border-border-subtle bg-canvas px-3 py-4 md:block"
  >
    <nav className="flex flex-col gap-5">
      <SideRailGroup heading="Explore" items={exploreItems} />
      <SideRailGroup heading="Genomes" items={genomeItems} />
      <SideRailGroup heading="Tools" items={toolItems} />
      <SideRailGroup heading="Data" items={dataItems} />
    </nav>
  </aside>
);

export default SideRail;
