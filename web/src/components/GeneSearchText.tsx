import type { ReactElement } from "react";

const GeneSearchText = (): ReactElement => (
  <div>
    <h2 className="text-2xl font-semibold">Genes</h2>
    <p className="mt-2 max-w-2xl text-sm leading-6 text-zinc-600">
      Search by gene ID, symbol, or locus tag, then open a dedicated page for coordinates,
      transcripts, exons, and source attributes.
    </p>
  </div>
);

export default GeneSearchText;
