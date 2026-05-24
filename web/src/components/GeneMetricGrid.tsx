import type { GeneRecord } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import geneFormat from "@/lib/geneFormat";
import Metric from "@/components/Metric";

const GeneMetricGrid = (props: { geneRecord: GeneRecord }): ReactElement => (
  <div className="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
    <Metric label="Sequence" value={props.geneRecord.gene.sequence_name} />
    <Metric label="Region" value={geneFormat.formatRegion(props.geneRecord.gene)} />
    <Metric label="Strand" value={geneFormat.formatStrand(props.geneRecord.gene.strand)} />
    <Metric label="Transcripts" value={String(props.geneRecord.transcripts.length)} />
  </div>
);

export default GeneMetricGrid;
