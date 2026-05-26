import type { KeggPathwayKoEntry, KeggPathwayDetail } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import KeggPathwayExpressionHeatmap from "@/features/kegg/components/KeggPathwayExpressionHeatmap";
import KeggPathwayKoCard from "@/features/kegg/components/KeggPathwayKoCard";
import EmptyState from "@/shared/ui/EmptyState";

const zero = 0;

const renderEntries = (kos: KeggPathwayKoEntry[]): ReactElement => {
  if (kos.length === zero) {
    return (
      <EmptyState
        description="The catalog has no KO links registered against this pathway."
        title="No KOs in this dataset"
      />
    );
  }
  return (
    <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
      {kos.map((entry) => (
        <KeggPathwayKoCard entry={entry} key={entry.ko} />
      ))}
    </div>
  );
};

const KeggPathwayBody = (props: { data: KeggPathwayDetail }): ReactElement => (
  <>
    <KeggPathwayExpressionHeatmap data={props.data} />
    {renderEntries(props.data.kos)}
  </>
);

export default KeggPathwayBody;
