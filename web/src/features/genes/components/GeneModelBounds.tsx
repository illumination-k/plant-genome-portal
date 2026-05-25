import type { Gene } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import geneFormat from "@/shared/lib/geneFormat";

const oneBasedOffset = 1;

const GeneModelBounds = (props: { gene: Gene }): ReactElement => (
  <div className="flex items-center justify-between gap-4 text-xs font-medium text-text-muted">
    <span>{geneFormat.formatPosition(props.gene.region.start + oneBasedOffset)}</span>
    <span>{geneFormat.formatPosition(props.gene.region.end)}</span>
  </div>
);

export default GeneModelBounds;
