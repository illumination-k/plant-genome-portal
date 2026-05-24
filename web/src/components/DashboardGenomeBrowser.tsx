import GenomeBrowser from "@/components/GenomeBrowser";
import type { ReactElement } from "react";

const DashboardGenomeBrowser = (): ReactElement => (
  <div className="col-span-12 rounded-lg border border-zinc-200 bg-white p-6">
    <h2 className="text-base font-semibold">Genome browser</h2>
    <p className="mb-4 mt-1 text-sm text-zinc-600">
      Explore the default assembly with the gene annotation track.
    </p>
    <GenomeBrowser />
  </div>
);

export default DashboardGenomeBrowser;
