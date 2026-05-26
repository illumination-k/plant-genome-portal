import type { GeneRecord } from "@/api/client/types.gen";
import type { UseQueryResult } from "@tanstack/react-query";
import type { ReactElement } from "react";
import GeneBackLink from "@/features/genes/components/GeneBackLink";
import GeneDetailLoading from "@/features/genes/components/GeneDetailLoading";
import GeneDetailTabs from "@/features/genes/components/GeneDetailTabs";
import GeneHeader from "@/shared/bio/GeneHeader";
import EmptyState from "@/shared/ui/EmptyState";
import ErrorState from "@/shared/ui/ErrorState";
import geneRecordUtils from "@/shared/lib/geneRecordUtils";

const GeneDetailState = (props: {
  geneId: string;
  geneQuery: UseQueryResult<GeneRecord, unknown>;
}): ReactElement => {
  if (props.geneId === "") {
    return <EmptyState description="Open a gene from the genes page." title="Missing gene ID" />;
  }

  if (props.geneQuery.isLoading) {
    return <GeneDetailLoading geneId={props.geneId} />;
  }

  if (props.geneQuery.error) {
    return (
      <ErrorState
        detail={geneRecordUtils.errorMessage(props.geneQuery.error)}
        title={`Gene ${props.geneId} could not be loaded`}
      />
    );
  }

  if (!props.geneQuery.data) {
    return <EmptyState description={props.geneId} title="Gene not found" />;
  }

  return (
    <section className="flex flex-col gap-6">
      <GeneBackLink />
      <GeneHeader gene={props.geneQuery.data.gene} />
      <GeneDetailTabs geneRecord={props.geneQuery.data} />
    </section>
  );
};

export default GeneDetailState;
