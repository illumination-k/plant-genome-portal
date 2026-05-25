import type { Gene } from "@/api/client/types.gen";
import GeneResultsContent from "@/features/genes/components/GeneResultsContent";
import GeneResultsHeader from "@/features/genes/components/GeneResultsHeader";
import type { ReactElement } from "react";

const GeneResultsPanel = (props: {
  error: Error | unknown;
  genes: Gene[];
  isFetching: boolean;
}): ReactElement => (
  <div className="col-span-12 overflow-hidden rounded-lg border border-border-subtle bg-surface">
    <GeneResultsHeader isFetching={props.isFetching} />
    <GeneResultsContent error={props.error} genes={props.genes} />
  </div>
);

export default GeneResultsPanel;
