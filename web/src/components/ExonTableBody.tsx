import type { Exon } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import geneRecordUtils from "@/lib/geneRecordUtils";
import ExonRow from "@/components/ExonRow";

const ExonTableBody = (props: { exons: Exon[] }): ReactElement => (
  <tbody className="divide-y divide-zinc-200">
    {props.exons.map((exon) => (
      <ExonRow exon={exon} key={geneRecordUtils.exonKey(exon)} />
    ))}
  </tbody>
);

export default ExonTableBody;
