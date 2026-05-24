import type { ReactElement } from "react";

const swatch = "inline-block h-2.5 w-4 rounded-sm align-middle";

const GeneStructureLegend = (): ReactElement => (
  <div className="mt-3 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-zinc-600">
    <span>
      <span className={`${swatch} border border-emerald-700 bg-emerald-200`} /> Exon (UTR)
    </span>
    <span>
      <span className={`${swatch} bg-emerald-700`} /> CDS
    </span>
    <span>
      <span className="inline-block h-px w-5 align-middle bg-zinc-400" /> Intron
    </span>
  </div>
);

export default GeneStructureLegend;
