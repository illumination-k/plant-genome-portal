import type { Gene } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import geneFormat from "@/lib/geneFormat";
import GeneTable from "@/components/GeneTable";
import StatusMessage from "@/components/StatusMessage";

const emptyCount = 0;

const GeneResultsContent = (props: { error: Error | unknown; genes: Gene[] }): ReactElement => {
  if (props.error) {
    return (
      <StatusMessage detail={geneFormat.getErrorMessage(props.error)} title="Gene search failed" />
    );
  }

  if (props.genes.length === emptyCount) {
    return (
      <StatusMessage detail="Try another gene ID, symbol, or locus tag." title="No genes found" />
    );
  }

  return <GeneTable genes={props.genes} />;
};

export default GeneResultsContent;
