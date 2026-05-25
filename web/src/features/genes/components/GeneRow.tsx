import type { Gene } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import CoordinateRange from "@/shared/bio/CoordinateRange";
import GeneTitleCell from "@/features/genes/components/GeneTitleCell";
import StrandBadge from "@/shared/bio/StrandBadge";

const oneBasedOffset = 1;

const GeneRow = (props: { gene: Gene }): ReactElement => (
  <tr className="hover:bg-surface-muted">
    <GeneTitleCell gene={props.gene} />
    <td className="px-4 py-3 font-mono text-[12px] text-text-muted">
      {props.gene.assembly_accession}
    </td>
    <td className="px-4 py-3">
      <CoordinateRange
        chr={props.gene.sequence_name}
        end={props.gene.region.end}
        start={props.gene.region.start + oneBasedOffset}
      />
    </td>
    <td className="px-4 py-3">
      <StrandBadge strand={props.gene.strand} />
    </td>
    <td className="px-4 py-3 text-[13px] text-text-muted">{props.gene.feature_type}</td>
  </tr>
);

export default GeneRow;
