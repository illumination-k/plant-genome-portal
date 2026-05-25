import type { GeneRecord } from "@/api/client/types.gen";
import GeneModelBounds from "@/features/genes/components/GeneModelBounds";
import GeneModelCaption from "@/features/genes/components/GeneModelCaption";
import GeneStructureViz from "@/features/genes/components/GeneStructureViz";
import type { ReactElement } from "react";

const GeneModel = (props: { geneRecord: GeneRecord }): ReactElement => (
  <div className="mt-6 rounded-lg border border-border-subtle bg-surface-muted p-4">
    <GeneModelBounds gene={props.geneRecord.gene} />
    <GeneStructureViz geneRecord={props.geneRecord} />
    <GeneModelCaption gene={props.geneRecord.gene} />
  </div>
);

export default GeneModel;
