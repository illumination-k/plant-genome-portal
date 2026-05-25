import type { ReactElement } from "react";
import ResultCountBadge from "@/features/genes/components/ResultCountBadge";
import GeneSearchText from "@/features/genes/components/GeneSearchText";

const GeneSearchHeader = (props: { resultCount: number }): ReactElement => (
  <div className="flex flex-wrap items-start justify-between gap-4">
    <GeneSearchText />
    <ResultCountBadge resultCount={props.resultCount} />
  </div>
);

export default GeneSearchHeader;
