import type { Exon } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import geneRecordUtils from "@/shared/lib/geneRecordUtils";
import ExonRow from "@/features/genes/components/ExonRow";

const ExonTableBody = (props: { exons: Exon[] }): ReactElement => (
  <tbody className="divide-y divide-border-subtle">
    {props.exons.map((exon) => (
      <ExonRow exon={exon} key={geneRecordUtils.exonKey(exon)} />
    ))}
  </tbody>
);

export default ExonTableBody;
