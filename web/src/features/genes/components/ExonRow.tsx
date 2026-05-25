import type { Exon } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import geneFormat from "@/shared/lib/geneFormat";

const ExonRow = (props: { exon: Exon }): ReactElement => (
  <tr>
    <td className="px-4 py-3 font-medium text-text">{props.exon.transcript_id}</td>
    <td className="px-4 py-3 text-text-muted">{props.exon.sequence_name}</td>
    <td className="px-4 py-3 text-text-muted">
      {geneFormat.formatLocation(props.exon.sequence_name, props.exon.region)}
    </td>
    <td className="px-4 py-3 text-text-muted">{geneFormat.formatStrand(props.exon.strand)}</td>
  </tr>
);

export default ExonRow;
