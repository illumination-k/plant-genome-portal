import type { KeggGeneSummary } from "@/api/client/types.gen";
import type { ReactElement } from "react";

const renderSymbol = (symbol: string | null | undefined): ReactElement | false => {
  if (symbol === null || symbol === undefined || symbol === "") {
    return false;
  }
  return <span className="text-[12px] text-text-muted">{symbol}</span>;
};

const renderLocusTag = (locusTag: string | null | undefined): ReactElement | false => {
  if (locusTag === null || locusTag === undefined || locusTag === "") {
    return false;
  }
  return <span className="text-[12px] text-text-subtle">[{locusTag}]</span>;
};

const KeggPathwayGeneRow = (props: { gene: KeggGeneSummary }): ReactElement => (
  <li className="flex flex-wrap items-baseline gap-2 py-1">
    <a
      className="font-mono text-[12px] text-text underline-offset-2 hover:underline"
      href={`/genes/${props.gene.id}`}
    >
      {props.gene.id}
    </a>
    {renderSymbol(props.gene.symbol)}
    {renderLocusTag(props.gene.locus_tag)}
  </li>
);

export default KeggPathwayGeneRow;
