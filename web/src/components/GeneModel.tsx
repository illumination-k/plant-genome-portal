import type { GeneRecord } from "@/api/client/types.gen";
import GeneModelBounds from "@/components/GeneModelBounds";
import GeneModelCaption from "@/components/GeneModelCaption";
import GeneStructureViz from "@/components/GeneStructureViz";
import type { ReactElement } from "react";

const GeneModel = (props: { geneRecord: GeneRecord }): ReactElement => (
  <div className="mt-6 rounded-lg border border-zinc-200 bg-zinc-50 p-4">
    <GeneModelBounds gene={props.geneRecord.gene} />
    <GeneStructureViz geneRecord={props.geneRecord} />
    <GeneModelCaption gene={props.geneRecord.gene} />
  </div>
);

export default GeneModel;
