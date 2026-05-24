import type { Gene } from "@/api/client/types.gen";
import GeneResultsContent from "@/components/GeneResultsContent";
import GeneResultsHeader from "@/components/GeneResultsHeader";
import type { ReactElement } from "react";

const GeneResultsPanel = (props: {
  error: Error | unknown;
  genes: Gene[];
  isFetching: boolean;
}): ReactElement => (
  <div className="col-span-12 overflow-hidden rounded-lg border border-zinc-200 bg-white">
    <GeneResultsHeader isFetching={props.isFetching} />
    <GeneResultsContent error={props.error} genes={props.genes} />
  </div>
);

export default GeneResultsPanel;
