import type { GeneRecord } from "@/api/client/types.gen";
import type { ReactElement, ReactNode } from "react";
import { useMemo } from "react";
import geneCoreDetailTabs from "@/features/genes/components/geneCoreDetailTabs";
import geneOmicsDetailTabs from "@/features/genes/components/geneOmicsDetailTabs";
import Tabs from "@/shared/ui/Tabs";

type DetailTab = {
  label: string;
  panel: ReactNode;
  value: string;
};

const CORE_START_INDEX = 0;
const CORE_OMICS_INSERT_INDEX = 2;

const buildTabs = (geneRecord: GeneRecord): DetailTab[] => {
  const coreTabs = geneCoreDetailTabs(geneRecord);

  return [
    ...coreTabs.slice(CORE_START_INDEX, CORE_OMICS_INSERT_INDEX),
  ...geneOmicsDetailTabs(geneRecord),
    ...coreTabs.slice(CORE_OMICS_INSERT_INDEX),
  ];
};

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
