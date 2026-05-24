import type { Gene } from "@/api/client/types.gen";
import GeneHeroTitle from "@/components/GeneHeroTitle";
import type { ReactElement } from "react";

const GeneHeroHeader = (props: { gene: Gene }): ReactElement => (
  <div className="flex flex-wrap items-start justify-between gap-4">
    <GeneHeroTitle gene={props.gene} />
    <span className="rounded-md bg-sky-100 px-3 py-2 text-sm font-semibold text-sky-800">
      {props.gene.assembly_accession}
    </span>
  </div>
);

export default GeneHeroHeader;
