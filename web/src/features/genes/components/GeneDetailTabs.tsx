import type { GeneRecord } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import { useCallback } from "react";
import { useSearchParams } from "react-router";
import { type InferOutput, picklist } from "valibot";
import GeneDetailTabsView from "@/features/genes/components/GeneDetailTabsView";
import useValidatedSearchParam from "@/shared/lib/useValidatedSearchParam";

const TAB_PARAM = "tab";
const DEFAULT_TAB = "overview";

const tabSchema = picklist([
  "overview",
  "annotation",
  "expression",
  "orthologs",
  "epigenome",
  "sequence",
  "transcripts",
  "browser",
]);

type TabValue = InferOutput<typeof tabSchema>;

const GeneDetailTabs = (props: { geneRecord: GeneRecord }): ReactElement => {
  const value: TabValue = useValidatedSearchParam(TAB_PARAM, tabSchema, DEFAULT_TAB);
  const [, setSearchParams] = useSearchParams();

  const onValueChange = useCallback(
    (next: string): void => {
      setSearchParams(
        (current) => {
          const updated = new URLSearchParams(current);
          if (next === DEFAULT_TAB) {
            updated.delete(TAB_PARAM);
          } else {
            updated.set(TAB_PARAM, next);
          }
          return updated;
        },
        { replace: true },
      );
    },
    [setSearchParams],
  );

  return (
    <GeneDetailTabsView geneRecord={props.geneRecord} onValueChange={onValueChange} value={value} />
  );
};

export default GeneDetailTabs;
