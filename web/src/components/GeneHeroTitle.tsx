import type { Gene } from "@/api/client/types.gen";
import type { ReactElement } from "react";

const getTitle = (gene: Gene): string => gene.symbol ?? gene.id;

const GeneHeroTitle = (props: { gene: Gene }): ReactElement => (
  <div>
    <p className="text-sm font-medium text-text-muted">{props.gene.feature_type}</p>
    <h2 className="mt-1 text-3xl font-semibold text-text">{getTitle(props.gene)}</h2>
    <p className="mt-2 text-sm text-text-muted">{props.gene.id}</p>
  </div>
);

export default GeneHeroTitle;
