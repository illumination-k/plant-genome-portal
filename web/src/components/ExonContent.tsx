import type { Exon } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import ExonTable from "@/components/ExonTable";
import EmptyState from "@/ui/EmptyState";

const emptyCount = 0;

const ExonContent = (props: { exons: Exon[] }): ReactElement => {
  if (props.exons.length === emptyCount) {
    return (
      <EmptyState description="This record does not include exon annotations." title="No exons" />
    );
  }

  return <ExonTable exons={props.exons} />;
};

export default ExonContent;
