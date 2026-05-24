import type { Gene } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import geneFormat from "@/lib/geneFormat";

const oneBasedOffset = 1;

const GeneModelCaption = (props: { gene: Gene }): ReactElement => (
  <p className="mt-3 text-sm text-text-muted">
    {props.gene.sequence_name}:{geneFormat.formatPosition(props.gene.region.start + oneBasedOffset)}
    -{geneFormat.formatPosition(props.gene.region.end)}
  </p>
);

export default GeneModelCaption;
