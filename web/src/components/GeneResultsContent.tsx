import type { Gene } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import geneFormat from "@/lib/geneFormat";
import GeneTable from "@/components/GeneTable";
import EmptyState from "@/ui/EmptyState";
import ErrorState from "@/ui/ErrorState";

const emptyCount = 0;

const GeneResultsContent = (props: { error: Error | unknown; genes: Gene[] }): ReactElement => {
  if (props.error) {
    return (
      <ErrorState detail={geneFormat.getErrorMessage(props.error)} title="Gene search failed" />
    );
  }

  if (props.genes.length === emptyCount) {
    return (
      <EmptyState description="Try another gene ID, symbol, or locus tag." title="No genes found" />
    );
  }

  return <GeneTable genes={props.genes} />;
};

export default GeneResultsContent;
