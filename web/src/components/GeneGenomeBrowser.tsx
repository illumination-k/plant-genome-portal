import type { Gene } from "@/api/client/types.gen";
import GenomeBrowser from "@/components/GenomeBrowser";
import type { ReactElement } from "react";

const minFlankBp = 1000;
const flankFraction = 0.5;
const minGeneLength = 1;
const zeroBased = 0;
const oneBasedOffset = 1;

const buildLocation = (gene: Gene): string => {
  const start0 = gene.region.start;
  const end0 = gene.region.end;
  const length = Math.max(end0 - start0, minGeneLength);
  const flank = Math.max(Math.floor(length * flankFraction), minFlankBp);
  const flankedStart0 = Math.max(start0 - flank, zeroBased);
  const flankedEnd0 = end0 + flank;
  const startOneBased = flankedStart0 + oneBasedOffset;
  return `${gene.sequence_name}:${startOneBased}..${flankedEnd0}`;
};

const GeneGenomeBrowser = (props: { gene: Gene }): ReactElement => (
  <div className="mt-6">
    <h3 className="text-base font-semibold">Genome browser</h3>
    <p className="mb-3 mt-1 text-sm text-zinc-600">
      Region around {props.gene.sequence_name} containing this gene.
    </p>
    <GenomeBrowser
      accession={props.gene.assembly_accession}
      location={buildLocation(props.gene)}
    />
  </div>
);

export default GeneGenomeBrowser;
