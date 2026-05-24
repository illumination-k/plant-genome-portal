import GeneSearchForm from "@/components/GeneSearchForm";
import GeneSearchHeader from "@/components/GeneSearchHeader";
import type { ReactElement } from "react";

const GeneSearchPanel = (props: { resultCount: number; searchText: string }): ReactElement => (
  <div className="col-span-12 rounded-lg border border-border-subtle bg-surface p-6">
    <GeneSearchHeader resultCount={props.resultCount} />
    <GeneSearchForm searchText={props.searchText} />
  </div>
);

export default GeneSearchPanel;
