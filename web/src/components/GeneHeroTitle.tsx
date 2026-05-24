import type { Gene } from "@/api/client/types.gen";
import type { ReactElement } from "react";

const getTitle = (gene: Gene): string => gene.symbol ?? gene.id;

const GeneHeroTitle = (props: { gene: Gene }): ReactElement => (
  <div>
    <p className="text-sm font-medium text-zinc-500">{props.gene.feature_type}</p>
    <h2 className="mt-1 text-3xl font-semibold text-zinc-950">{getTitle(props.gene)}</h2>
    <p className="mt-2 text-sm text-zinc-600">{props.gene.id}</p>
  </div>
);

export default GeneHeroTitle;
