import type { GeneRecord } from "@/api/client/types.gen";
import type { ReactNode } from "react";
import GeneAnnotationTab from "@/features/genes/components/GeneAnnotationTab";
import GeneBrowserTab from "@/features/genes/components/GeneBrowserTab";
import GeneOrthologsTab from "@/features/genes/components/GeneOrthologsTab";
import GeneOverviewTab from "@/features/genes/components/GeneOverviewTab";
import GeneSequenceTab from "@/features/genes/components/GeneSequenceTab";
import GeneTranscriptsTab from "@/features/genes/components/GeneTranscriptsTab";

type GeneDetailTab = {
  label: string;
  panel: ReactNode;
  value: string;
};

const geneCoreDetailTabs = (geneRecord: GeneRecord): GeneDetailTab[] => [
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
    label: "Orthologs",
    panel: <GeneOrthologsTab geneRecord={geneRecord} />,
    value: "orthologs",
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

export default geneCoreDetailTabs;
