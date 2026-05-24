import type { Exon } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import geneFormat from "@/lib/geneFormat";

const ExonRow = (props: { exon: Exon }): ReactElement => (
  <tr>
    <td className="px-4 py-3 font-medium text-zinc-900">{props.exon.transcript_id}</td>
    <td className="px-4 py-3 text-zinc-600">{props.exon.sequence_name}</td>
    <td className="px-4 py-3 text-zinc-600">
      {geneFormat.formatLocation(props.exon.sequence_name, props.exon.region)}
    </td>
    <td className="px-4 py-3 text-zinc-600">{geneFormat.formatStrand(props.exon.strand)}</td>
  </tr>
);

export default ExonRow;
