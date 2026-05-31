import type { GeneRecord } from "@/api/client/types.gen";
import type { ReactNode } from "react";
import GeneEpigenomeTab from "@/features/epigenome/components/GeneEpigenomeTab";
import GeneExpressionTab from "@/features/expression/components/GeneExpressionTab";

type GeneDetailTab = {
  label: string;
  panel: ReactNode;
  value: string;
};

const geneOmicsDetailTabs = (geneRecord: GeneRecord): GeneDetailTab[] => [
  {
    label: "Expression",
    panel: <GeneExpressionTab geneRecord={geneRecord} />,
    value: "expression",
  },
  {
    label: "Epigenome",
    panel: <GeneEpigenomeTab geneRecord={geneRecord} />,
    value: "epigenome",
  },
];

export default geneOmicsDetailTabs;
