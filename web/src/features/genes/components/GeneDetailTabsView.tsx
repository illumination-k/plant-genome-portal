import type { GeneRecord } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import { useMemo } from "react";
import GeneAnnotationTab from "@/features/genes/components/GeneAnnotationTab";
import GeneBrowserTab from "@/features/genes/components/GeneBrowserTab";
import GeneExpressionTab from "@/features/expression/components/GeneExpressionTab";
import GeneOverviewTab from "@/features/genes/components/GeneOverviewTab";
import GeneSequenceTab from "@/features/genes/components/GeneSequenceTab";
import GeneTranscriptsTab from "@/features/genes/components/GeneTranscriptsTab";
import Tabs from "@/shared/ui/Tabs";

const GeneDetailTabsView = (props: {
  geneRecord: GeneRecord;
  onValueChange: (value: string) => void;
  value: string;
}): ReactElement => {
  const tabs = useMemo(
    () => [
      {
        label: "Overview",
        panel: <GeneOverviewTab geneRecord={props.geneRecord} />,
        value: "overview",
      },
      {
        label: "Annotation",
        panel: <GeneAnnotationTab gene={props.geneRecord.gene} />,
        value: "annotation",
      },
      {
        label: "Expression",
        panel: <GeneExpressionTab geneRecord={props.geneRecord} />,
        value: "expression",
      },
      {
        label: "Sequence",
        panel: <GeneSequenceTab geneRecord={props.geneRecord} />,
        value: "sequence",
      },
      {
        label: "Transcripts",
        panel: <GeneTranscriptsTab geneRecord={props.geneRecord} />,
        value: "transcripts",
      },
      {
        label: "Browser",
        panel: <GeneBrowserTab geneRecord={props.geneRecord} />,
        value: "browser",
      },
    ],
    [props.geneRecord],
  );

  return (
    <Tabs
      ariaLabel="Gene detail sections"
      onValueChange={props.onValueChange}
      tabs={tabs}
      value={props.value}
    />
  );
};

export default GeneDetailTabsView;
