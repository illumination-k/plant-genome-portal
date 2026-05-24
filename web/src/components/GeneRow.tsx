import type { Gene } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import geneFormat from "@/lib/geneFormat";
import GeneTitleCell from "@/components/GeneTitleCell";

const GeneRow = (props: { gene: Gene }): ReactElement => (
  <tr>
    <GeneTitleCell gene={props.gene} />
    <td className="px-4 py-3 text-text-muted">{props.gene.assembly_accession}</td>
    <td className="px-4 py-3 text-text-muted">
      {geneFormat.formatLocation(props.gene.sequence_name, props.gene.region)}
    </td>
    <td className="px-4 py-3 text-text-muted">{geneFormat.formatStrand(props.gene.strand)}</td>
    <td className="px-4 py-3 text-text-muted">{props.gene.feature_type}</td>
  </tr>
);

export default GeneRow;
