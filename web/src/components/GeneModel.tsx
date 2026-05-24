import type { Gene } from "@/api/client/types.gen";
import GeneModelBounds from "@/components/GeneModelBounds";
import GeneModelCaption from "@/components/GeneModelCaption";
import type { ReactElement } from "react";

const GeneModel = (props: { gene: Gene }): ReactElement => (
  <div className="mt-6 rounded-lg border border-zinc-200 bg-zinc-50 p-4">
    <GeneModelBounds gene={props.gene} />
    <div className="mt-3 h-4 rounded-full bg-zinc-200 p-1">
      <div className="h-2 rounded-full bg-emerald-700" />
    </div>
    <GeneModelCaption gene={props.gene} />
  </div>
);

export default GeneModel;
