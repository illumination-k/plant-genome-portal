/* oxlint-disable max-lines-per-function */
import type { GeneRecord } from "@/api/client/types.gen";
import type { ReactElement, ReactNode } from "react";
import { useMemo } from "react";
import GeneAnnotationTab from "@/features/genes/components/GeneAnnotationTab";
import GeneBrowserTab from "@/features/genes/components/GeneBrowserTab";
import GeneEpigenomeTab from "@/features/epigenome/components/GeneEpigenomeTab";
import GeneExpressionTab from "@/features/expression/components/GeneExpressionTab";
import GeneOrthologsTab from "@/features/genes/components/GeneOrthologsTab";
import GeneOverviewTab from "@/features/genes/components/GeneOverviewTab";
import GeneSequenceTab from "@/features/genes/components/GeneSequenceTab";
import GeneTranscriptsTab from "@/features/genes/components/GeneTranscriptsTab";
import Tabs from "@/shared/ui/Tabs";

type Tab = {
  label: string;
  panel: ReactNode;
  value: string;
};

const buildTabs = (geneRecord: GeneRecord): Tab[] => [
  {
    label: "Overview",
    panel: <GeneOverviewTab geneRecord={geneRecord} />,
    value: "overview",
  },
  {
    label: "Annotation",
    panel: <GeneAnnotationTab gene={geneRecord.gene} />,
    value: "annotation",
  },
  {
    label: "Expression",
    panel: <GeneExpressionTab geneRecord={geneRecord} />,
    value: "expression",
  },
  {
    label: "Orthologs",
    panel: <GeneOrthologsTab geneRecord={geneRecord} />,
    value: "orthologs",
  },
  {
    label: "Epigenome",
    panel: <GeneEpigenomeTab geneRecord={geneRecord} />,
    value: "epigenome",
  },
  {
    label: "Sequence",
    panel: <GeneSequenceTab geneRecord={geneRecord} />,
    value: "sequence",
  },
  {
    label: "Transcripts",
    panel: <GeneTranscriptsTab geneRecord={geneRecord} />,
    value: "transcripts",
  },
  {
    label: "Browser",
    panel: <GeneBrowserTab geneRecord={geneRecord} />,
    value: "browser",
  },
];

const GeneDetailTabsView = (props: {
  geneRecord: GeneRecord;
  onValueChange: (value: string) => void;
  value: string;
}): ReactElement => {
  const tabs = useMemo(() => buildTabs(props.geneRecord), [props.geneRecord]);

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
